use pyo3::prelude::*;

#[pymodule]
fn diggity(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(path, m)?)?;
    Ok(())
}

/// Tries to extract the value of nested structs by a specified path.
/// If unsuccessful, it returns a None or a default value if provided.
/// - `obj`: The object with nested structures from which to extract values.
/// - `path`: A string representing the path to the desired value, using the specified separator.
/// - `default`: An optional default value to return if the path is not found.
/// - `sep`: An optional string to specify the separator used in the path (default is ".").
#[pyfunction]
#[pyo3(signature = (obj, path, r#default=None, sep = "."))]
fn path(
    py: Python,
    obj: &Bound<'_, PyAny>,
    path: &str,
    r#default: Option<&Bound<'_, PyAny>>,
    sep: &str,
) -> PyResult<PyObject> {
    let keys = path.split(sep);
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
                } else if let Some(value) = r#default {
                    return Ok(value.clone().unbind());
                } else {
                    return Ok(py.None());
                }
            }
        }
    }
    Ok(obj.clone().unbind())
}

// #[pyfunction]
// #[pyo3(signature = (obj, path, r#default=None))]
// fn path(py: Python, obj: &PyAny, path: &str, r#default: Option<&PyAny>) -> PyResult<PyObject> {
//     let mut current_obj = obj.to_object(py);
//     for key in path.split('.') {
//         match current_obj.getattr(key) {
//             Ok(value) => current_obj = value.to_object(py),
//             Err(_) => {
//                 if let Ok(index) = key.parse::<usize>() {
//                     current_obj = match current_obj.get_item(index) {
//                         Ok(value) => value.to_object(py),
//                         Err(_) => match r#default {
//                             Some(default_val) => return Ok(default_val.to_object(py)),
//                             None => return Ok(py.none()),
//                         },
//                     };
//                     continue;
//                 }
//                 match current_obj.get_item(key) {
//                     Ok(value) => current_obj = value.to_object(py),
//                     Err(_) => {
//                         return match r#default {
//                             Some(default_val) => Ok(default_val.to_object(py)),
//                             None => Ok(py.none()),
//                         }
//                     }
//                 }
//             }
//         }
//     }
//     Ok(current_obj)
// }

// /// Tries to extract the value of nested structs by a specified path.
// /// If unsuccessful, it returns a None or a default value if provided.
// #[pyfunction]
// #[pyo3(signature = (obj, path, r#default=None))]
// fn path(
//     py: Python,
//     obj: &PyAny,
//     path: &str,
//     r#default: Option<&PyAny>,
// ) -> PyResult<PyObject> {
//     let mut current_obj = obj.to_object(py);
//
//     // Use an iterator to traverse the keys from the path
//     let result = path.split('.').try_fold(current_obj, |obj, key| {
//         // Attempt to access the attribute or the item
//         match obj.getattr(key) {
//             Ok(value) => Ok(value.to_object(py)),
//             Err(_) => {
//                 // Try to parse the key as an index
//                 match key.parse::<usize>() {
//                     Ok(index) => match obj.get_item(index) {
//                         Ok(value) => Ok(value.to_object(py)),
//                         Err(_) => return handle_default(py, r#default),
//                     },
//                     Err(_) => match obj.get_item(key) {
//                         Ok(value) => Ok(value.to_object(py)),
//                         Err(_) => return handle_default(py, r#default),
//                     },
//                 }
//             }
//         }
//     });
//
//     // Return the final result or handle errors
//     match result {
//         Ok(final_obj) => Ok(final_obj),
//         Err(_) => Ok(py.none()), // If any steps fail, return None
//     }
// }
//
// fn handle_default(py: Python, r#default: Option<&PyAny>) -> PyResult<PyObject> {
//     if let Some(default_value) = r#default {
//         Ok(default_value.to_object(py))
//     } else {
//         Ok(py.none())
//     }
// }
