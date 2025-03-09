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
    exceptions::PyValueError,
    prelude::*,
    types::{PyAny, PyString, PyTuple},
};

use std::ops::ControlFlow;

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
fn coalesce(args: Bound<'_, PyTuple>) -> PyObject {
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
fn coalesce_logical(args: Bound<'_, PyTuple>) -> PyObject {
    args.iter()
        .find(|arg| arg.is_truthy().unwrap_or(false))
        .map_or_else(|| args.py().None(), |arg| arg.unbind())
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
) -> PyResult<PyObject> {
    let default_value = r#default;

    if args.is_empty() {
        return Ok(obj.unbind());
    }

    let value = args.iter().try_fold(obj, |acc, arg| {
        if let Ok(key) = arg.downcast::<PyString>() {
            acc.get_item(key).or_else(|_| acc.getattr(key)).map_or_else(
                |_| ControlFlow::Break(default_value),
                |v| ControlFlow::Continue(v),
            )
        } else {
            acc.get_item(arg).map_or_else(
                |_| ControlFlow::Break(default_value),
                |v| ControlFlow::Continue(v),
            )
        }
    });
    extract_control_flow_value(value, py)
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
/// # Examples
/// ```python
/// data = {"a": {"b": [{"c": 42}]}}
/// assert dig_path(data, "a.b.0.c") == 42
/// assert dig_path(data, "a.x.y", default=0) == 0
/// assert dig_path(data, "a/b/0/c", sep="/") == 42
/// ``
#[pyfunction]
#[pyo3(signature = (obj, path, r#default=None, sep = "."))]
fn dig_path(
    py: Python,
    obj: Bound<'_, PyAny>,
    path: &str,
    r#default: Option<&Bound<'_, PyAny>>,
    sep: &str,
) -> PyResult<PyObject> {
    let default_value = r#default;

    if path.is_empty() {
        return Ok(obj.unbind());
    }

    let value = path.split(sep).try_fold(obj, |acc, key| {
        acc.get_item(key)
            .or_else(|_| acc.getattr(key))
            .or_else(|_| {
                let index = key
                    .parse::<usize>()
                    .map_err(|_| PyValueError::new_err(py.None()))?;
                acc.get_item(index)
            })
            .map_or_else(
                |_| ControlFlow::Break(default_value),
                |v| ControlFlow::Continue(v),
            )
    });

    extract_control_flow_value(value, py)
}

#[inline]
fn extract_control_flow_value(
    value: ControlFlow<Option<&Bound<'_, PyAny>>, Bound<'_, PyAny>>,
    py: Python<'_>,
) -> PyResult<PyObject> {
    match value {
        ControlFlow::Continue(v) => Ok(v.unbind()),
        ControlFlow::Break(v) => Ok(v.into_pyobject(py)?.unbind()),
    }
}
