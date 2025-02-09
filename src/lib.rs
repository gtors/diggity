use pyo3::{
    exceptions::PyValueError,
    prelude::*,
    types::{PyAny, PyString, PyTuple},
};

use std::ops::ControlFlow;

#[pymodule]
fn diggity(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(dig, m)?)?;
    Ok(())
}

/// Tries to extract the value of nested structs by a specified path.
/// If unsuccessful, it returns a None or a default value if provided.
/// - `obj`: The object with nested structures from which to extract values.
/// - `path`: A string representing the path to the desired value, using the specified separator.
/// - `default`: An optional default value to return if the path is not found.
/// - `sep`: An optional string to specify the separator used in the path (default is ".").
#[pyfunction]
#[pyo3(signature = (obj, *args, path=None, r#default=None, sep = "."))]
fn dig(
    py: Python,
    obj: Bound<'_, PyAny>,
    args: &Bound<'_, PyTuple>,
    path: Option<&Bound<'_, PyString>>,
    r#default: Option<&Bound<'_, PyAny>>,
    sep: &str,
) -> PyResult<PyObject> {
    if !args.is_empty() {
        dig_args(obj, args, r#default, py)
    } else if let Some(path) = path {
        dig_path(obj, path, sep, r#default, py)
    } else {
        Ok(obj.unbind())
    }
}

fn dig_path(
    obj: Bound<'_, PyAny>,
    path: &Bound<'_, PyString>,
    sep: &str,
    default_value: Option<&Bound<'_, PyAny>>,
    py: Python,
) -> PyResult<PyObject> {
    let path_str = path.to_cow()?;

    if path_str.is_empty() {
        return Ok(obj.unbind());
    }

    let value = path_str.split(sep).try_fold(obj, |acc, key| {
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

fn dig_args(
    obj: Bound<'_, PyAny>,
    args: &Bound<'_, PyTuple>,
    default_value: Option<&Bound<'_, PyAny>>,
    py: Python,
) -> PyResult<PyObject> {
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
