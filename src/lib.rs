/// A utility module for working with nested Python objects and handling optional values.
///
/// This module provides functions to extract values from nested structures, handle `None` values,
/// and find the first truthy/not-None value in a sequence of arguments.
///
/// # Functions
/// - `dig`: Extracts values from nested structures using a sequence of keys or attributes.
/// - `dig_path`: Extracts values from nested structures using a path string with a specified separator.
/// - `coalesce`: Returns the first non-`None` value from a sequence of arguments.
/// - `coalesce_logical`: Returns the first truthy value from a sequence of arguments.
///
/// # Notes
/// - The `dig` function is inspired by Ruby's `Hash#dig` and `Array#dig` methods, providing
///   a safe way to access deeply nested values without raising errors for missing keys or indices
/// - The `dig` and `dig_path` functions are useful for traversing deeply nested dictionaries, sequences and objects.
/// - The `coalesce` and `coalesce_logical` functions are helpful for providing fallback values in cases
///   where `None` or falsy values are present.
use pyo3::{
    exceptions::{
        PyAttributeError, PyIndexError, PyKeyError, PyRuntimeError, PyTypeError, PyValueError,
    },
    ffi,
    prelude::*,
    types::{PyAny, PyDict, PyInt, PyList, PyString, PyTuple},
    PyTypeInfo,
};

/// Reports whether `obj` is exactly of type `T` (subclasses do not match).
///
/// Fast paths only apply to exact builtin types: for subclasses, custom
/// `__getitem__`/`__missing__`/`__getattr__` implementations must keep running,
/// so those go through the generic (exception-driven) path instead.
#[inline]
fn is_exact<T: PyTypeInfo>(py: Python<'_>, obj: &Bound<'_, PyAny>) -> bool {
    let ty = unsafe { ffi::Py_TYPE(obj.as_ptr()) };
    ty == T::type_object_raw(py)
}

/// Extracts the pending exception, falling back to a `RuntimeError` if the
/// error state was unexpectedly empty.
#[cold]
fn take_err(py: Python<'_>) -> PyErr {
    PyErr::take(py).unwrap_or_else(|| PyRuntimeError::new_err("Python error state was not set"))
}

/// Reports whether an error raised during a lookup step merely means that the
/// requested key/attribute does not exist.
///
/// Only these exceptions are treated as "not found"; anything else (for example
/// a `RuntimeError` raised from a user-defined `__getattr__` or `__getitem__`)
/// is a genuine error and must be propagated to the caller instead of being
/// silently replaced with the default value.
fn is_missing(py: Python<'_>, err: &PyErr) -> bool {
    // Fast path: the four builtin types cover virtually every real miss. The
    // type objects are statically cached by PyO3, so this is a few pointer
    // compares instead of up to four `PyErr_GivenExceptionMatches` walks.
    let ty = err.get_type(py).as_ptr() as *mut ffi::PyTypeObject;
    if ty == PyKeyError::type_object_raw(py)
        || ty == PyIndexError::type_object_raw(py)
        || ty == PyAttributeError::type_object_raw(py)
        || ty == PyTypeError::type_object_raw(py)
    {
        return true;
    }

    // Slow path: a user-defined exception subclassing one of the above.
    err.is_instance_of::<PyKeyError>(py)
        || err.is_instance_of::<PyIndexError>(py)
        || err.is_instance_of::<PyAttributeError>(py)
        || err.is_instance_of::<PyTypeError>(py)
}

/// Reads `dict[key]` without raising on a miss.
#[inline]
fn dict_get_item<'py>(
    py: Python<'py>,
    dict: &Bound<'py, PyAny>,
    key: &Bound<'py, PyAny>,
) -> PyResult<Option<Bound<'py, PyAny>>> {
    let item = unsafe { ffi::PyDict_GetItemWithError(dict.as_ptr(), key.as_ptr()) };
    if item.is_null() {
        // NULL without a pending exception simply means "key not present".
        if unsafe { ffi::PyErr_Occurred() }.is_null() {
            return Ok(None);
        }
        return Err(take_err(py));
    }
    Ok(Some(unsafe { Bound::from_borrowed_ptr(py, item) }))
}

/// Reads `seq[index]` from an exact `list` or `tuple` without raising.
///
/// Negative indices are resolved against the length, matching Python semantics.
/// Out-of-range indices yield `Ok(None)` rather than raising `IndexError`.
fn sequence_get_item<'py>(
    py: Python<'py>,
    seq: &Bound<'py, PyAny>,
    index: isize,
) -> PyResult<Option<Bound<'py, PyAny>>> {
    let is_list = is_exact::<PyList>(py, seq);
    let len = if is_list {
        unsafe { ffi::PyList_GET_SIZE(seq.as_ptr()) }
    } else {
        unsafe { ffi::PyTuple_GET_SIZE(seq.as_ptr()) }
    };

    let index = if index < 0 {
        index + len as isize
    } else {
        index
    };
    if index < 0 || index >= len as isize {
        return Ok(None);
    }

    let index = index as ffi::Py_ssize_t;
    let item = unsafe {
        if is_list {
            ffi::PyList_GetItem(seq.as_ptr(), index)
        } else {
            ffi::PyTuple_GetItem(seq.as_ptr(), index)
        }
    };

    if item.is_null() {
        // Unreachable for an exact list/tuple with an in-range index.
        return Err(take_err(py));
    }
    Ok(Some(unsafe { Bound::from_borrowed_ptr(py, item) }))
}

/// Performs a single traversal step.
///
/// String keys are looked up as an item first and as an attribute second, which
/// allows both mappings and plain objects to be traversed. Returns `Ok(None)`
/// when the key does not exist, and propagates errors raised by user code.
fn lookup<'py>(
    obj: &Bound<'py, PyAny>,
    py: Python<'py>,
    key: &Bound<'py, PyAny>,
) -> PyResult<Option<Bound<'py, PyAny>>> {
    // Fast path: an integer key on an exact list/tuple. Saves raising (and
    // formatting the message of) an `IndexError` on every miss.
    if key.is_instance_of::<PyInt>()
        && (is_exact::<PyList>(py, obj) || is_exact::<PyTuple>(py, obj))
    {
        if let Ok(index) = key.extract::<isize>() {
            return sequence_get_item(py, obj, index);
        }
    }

    // Fast path: a string key on an exact dict. Saves raising a `KeyError` on
    // every miss; the attribute fallback below still runs to keep `dig(d, "keys")`
    // resolving `dict.keys` as it always has.
    if is_exact::<PyDict>(py, obj) && key.is_instance_of::<PyString>() {
        if let Some(value) = dict_get_item(py, obj, key)? {
            return Ok(Some(value));
        }
        return lookup_attribute(obj, py, key);
    }

    let result = if key.is_instance_of::<PyString>() {
        let name = key.cast::<PyString>()?;
        match obj.get_item(name) {
            Ok(value) => Ok(value),
            Err(err) if is_missing(py, &err) => obj.getattr(name),
            Err(err) => return Err(err),
        }
    } else {
        obj.get_item(key)
    };

    match result {
        Ok(value) => Ok(Some(value)),
        Err(err) if is_missing(py, &err) => Ok(None),
        Err(err) => Err(err),
    }
}

/// Attribute-only lookup used as the fallback after a dict item miss.
fn lookup_attribute<'py>(
    obj: &Bound<'py, PyAny>,
    py: Python<'py>,
    key: &Bound<'py, PyAny>,
) -> PyResult<Option<Bound<'py, PyAny>>> {
    let name = key.cast::<PyString>()?;
    match obj.getattr(name) {
        Ok(value) => Ok(Some(value)),
        Err(err) if is_missing(py, &err) => Ok(None),
        Err(err) => Err(err),
    }
}

/// Performs a single traversal step for `dig_path`.
///
/// In addition to [`lookup`], a segment that parses as an integer is retried as
/// an integer index, so that paths such as `"0.c"` or `"-1.c"` work on sequences
/// and on mappings with integer keys.
fn lookup_segment<'py>(
    obj: &Bound<'py, PyAny>,
    py: Python<'py>,
    segment: &str,
) -> PyResult<Option<Bound<'py, PyAny>>> {
    // Fast path: exact list/tuple with a numeric segment. This is the single
    // hottest case and it previously cost two raised exceptions (a `TypeError`
    // from `get_item`, then an `AttributeError` from `getattr`) before the
    // integer fallback could even be attempted.
    if is_exact::<PyList>(py, obj) || is_exact::<PyTuple>(py, obj) {
        if let Ok(index) = segment.parse::<isize>() {
            return sequence_get_item(py, obj, index);
        }
    }

    let key = PyString::new(py, segment);

    // Fast path: exact dict. Both the item and the attribute lookup are tried
    // without raising a `KeyError`; a numeric segment then falls through to the
    // integer index below so that dicts keyed by `int` keep working.
    if is_exact::<PyDict>(py, obj) {
        if let Some(value) = dict_get_item(py, obj, &key)? {
            return Ok(Some(value));
        }
        if let Some(value) = lookup_attribute(obj, py, &key)? {
            return Ok(Some(value));
        }
    } else if let Some(value) = lookup(obj, py, key.as_any())? {
        return Ok(Some(value));
    }

    let Ok(index) = segment.parse::<isize>() else {
        return Ok(None);
    };

    match obj.get_item(index) {
        Ok(value) => Ok(Some(value)),
        Err(err) if is_missing(py, &err) => Ok(None),
        Err(err) => Err(err),
    }
}

#[pymodule]
fn diggity(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(dig, m)?)?;
    m.add_function(wrap_pyfunction!(dig_path, m)?)?;
    m.add_function(wrap_pyfunction!(coalesce, m)?)?;
    m.add_function(wrap_pyfunction!(coalesce_logical, m)?)?;
    Ok(())
}

/// Returns the first non-`None` value from the provided arguments.
///
/// This function iterates through the given arguments and returns the first one that is not `None`.
/// If all arguments are `None`, it returns `None`.
///
/// # Arguments
/// - `*args`: A variable number of arguments to check for the first non-`None` value.
///
/// # Returns
/// - The first non-`None` value if found.
/// - `None` if all arguments are `None`.
///
/// # Examples
/// ```python
/// assert coalesce(None, None, 42, None) == 42
/// assert coalesce(None, None, None) is None
/// ```
#[pyfunction]
#[pyo3(signature = (*args))]
fn coalesce(args: &Bound<'_, PyTuple>) -> Py<PyAny> {
    args.iter()
        .find(|arg| !arg.is_none())
        .map_or_else(|| args.py().None(), |arg| arg.unbind())
}

/// Returns the first truthy value from the provided arguments.
///
/// This function iterates through the given arguments and returns the first one that evaluates to `True`
/// in a boolean context. If all arguments are falsy, it returns `None`.
///
/// # Arguments
/// - `*args`: A variable number of arguments to check for the first truthy value.
///
/// # Returns
/// - The first truthy value if found.
/// - `None` if all arguments are falsy.
///
/// # Examples
/// ```python
/// assert coalesce_logical(None, False, 42, 0) == 42
/// assert coalesce_logical(None, False, 0, "") is None
/// ```
#[pyfunction]
#[pyo3(signature = (*args))]
fn coalesce_logical(args: &Bound<'_, PyTuple>) -> Py<PyAny> {
    for arg in args.iter() {
        if arg.is_truthy().unwrap_or(false) {
            return arg.unbind();
        }
    }
    args.py().None()
}

/// Tries to extract the value from a nested structure
///
/// This function traverses the given object using the keys or attributes provided in `args`.
/// If the path is not found, it returns `None` or a specified default value.
///
/// # Arguments
/// - `obj`: The object containing nested structures from which to extract values.
/// - `*args`: A variable number of keys or attributes to traverse the nested structure.
/// - `default`: An optional default value to return if the path is not found.
///
/// # Returns
/// - The value at the specified path if found.
/// - `None` or the provided default value if the path is not found.
///
/// # Errors
/// Errors raised by user code (for example from a custom `__getitem__` or
/// `__getattr__`) are propagated instead of being reported as a missing path.
/// Only `KeyError`, `IndexError`, `AttributeError` and `TypeError` are treated
/// as a missing path.
///
/// # Examples
/// ```python
/// data = {"a": {"b": {"c": 42}}}
/// assert dig(data, "a", "b", "c") == 42
/// assert dig(data, "a", "x", default=0) == 0
/// ```
#[pyfunction]
#[pyo3(signature = (obj, *args, r#default=None))]
fn dig(
    py: Python,
    obj: Bound<'_, PyAny>,
    args: &Bound<'_, PyTuple>,
    r#default: Option<&Bound<'_, PyAny>>,
) -> PyResult<Py<PyAny>> {
    if args.is_empty() {
        return Ok(obj.unbind());
    }

    let mut current = obj;
    for arg in args.iter() {
        match lookup(&current, py, &arg)? {
            Some(value) => current = value,
            None => return Ok(r#default.into_pyobject(py)?.unbind()),
        }
    }

    Ok(current.unbind())
}

/// Tries to extract the value from a nested structure within an object using a specified path.
///
/// This function traverses the given object using the keys or attributes specified in the `path` string,
/// split by the provided separator. If the path is not found, it returns `None` or a specified default value.
///
/// # Arguments
/// - `obj`: The object containing nested structures from which to extract values.
/// - `path`: A string representing the path to the desired value, with keys or attributes separated by `sep`.
/// - `default`: An optional default value to return if the path is not found.
/// - `sep`: An optional string to specify the separator used in the path (default is ".").
///
/// # Returns
/// - The value at the specified path if found.
/// - `None` or the provided default value if the path is not found.
///
/// # Errors
/// - `ValueError` if `sep` is an empty string.
/// - Errors raised by user code (for example from a custom `__getitem__` or
///   `__getattr__`) are propagated instead of being reported as a missing path.
///   Only `KeyError`, `IndexError`, `AttributeError` and `TypeError` are treated
///   as a missing path.
///
/// # Examples
/// ```python
/// data = {"a": {"b": [{"c": 42}]}}
/// assert dig_path(data, "a.b.0.c") == 42
/// assert dig_path(data, "a.x.y", default=0) == 0
/// assert dig_path(data, "a/b/0/c", sep="/") == 42
/// assert dig_path([{"a": 1}], "-1.a") == 1
/// ```
#[pyfunction]
#[pyo3(signature = (obj, path, r#default=None, sep = "."))]
fn dig_path(
    py: Python,
    obj: Bound<'_, PyAny>,
    path: &str,
    r#default: Option<&Bound<'_, PyAny>>,
    sep: &str,
) -> PyResult<Py<PyAny>> {
    if sep.is_empty() {
        return Err(PyValueError::new_err("`sep` must not be empty"));
    }

    if path.is_empty() {
        return Ok(obj.unbind());
    }

    let mut current = obj;
    for segment in path.split(sep) {
        match lookup_segment(&current, py, segment)? {
            Some(value) => current = value,
            None => return Ok(r#default.into_pyobject(py)?.unbind()),
        }
    }

    Ok(current.unbind())
}
