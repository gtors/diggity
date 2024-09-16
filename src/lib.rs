use pyo3::prelude::*;

/// A Python module implemented in Rust.
#[pymodule]
fn diggity(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(path, m)?)?;
    Ok(())
}

/// Tries to extract value of nested structs by specified path, if no luck, then None will be returned
#[pyfunction]
fn path(py: Python, obj: &Bound<'_, PyAny>, path: &str) -> PyResult<PyObject> {
    let keys = path.split(".");
    let mut obj = obj.clone();
    for key in keys {
        match obj.getattr(key) {
            Ok(value) => obj = value,
            Err(_) => {
                if let Ok(index) = key.parse::<usize>() {
                    if let Ok(value) = obj.get_item(index) {
                        obj = value;
                        continue;
                    }
                } else if let Ok(value) = obj.get_item(key) {
                    obj = value;
                    continue;
                } else {
                    return Ok(py.None());
                }
            }
        }
    }
    Ok(obj.clone().unbind())
}
