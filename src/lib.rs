use pyo3::{
    prelude::*,
    types::{PyString, PyTuple},
};

use std::ops::ControlFlow;

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
#[pyo3(signature = (obj, *args, path=None, r#default=None, sep = "."))]
fn dig(
    py: Python,
    obj: &Bound<'_, PyAny>,
    args: Bound<'_, PyTuple>,
    path: Option<&Bound<'_, PyString>>,
    r#default: Option<&Bound<'_, PyAny>>,
    sep: &str,
) -> PyResult<PyObject> {
    if args.len() > 0 {
        dig_args(py, obj, args)
    } else if let Some(path) = path {
        dig_path(py, obj, path, sep)
    } else {
        Ok(py.None())
    }
}

fn dig_path(
    py: Python,
    obj: &Bound<'_, PyAny>,
    path: &Bound<'_, PyString>,
    sep: &str,
) -> PyResult<PyObject> {
    let path = path.to_cow()?;

    if path == "" {
        return Ok(obj.into_py(py));
    }

    let keys = path.to_cow().map(|s| {
        s.split(sep)
            .try_fold(obj, |acc, key| match acc.getattr(key.into_py(py)) {
                Ok(value) => ControlFlow::Continue(value),
                Err(_) => {
                    if let Ok(index) = key.parse::<usize>() {
                        if let Ok(value) = acc.get_item(index) {
                            ControlFlow::Continue(value)
                        } else {
                            ControlFlow::Break(py.None())
                        }
                    } else if let Ok(value) = acc.get_item(key) {
                        ControlFlow::Continue(value)
                    } else {
                        ControlFlow::Break(acc)
                    }
                }
            })
    });

    // } else if let Some(value) = r#default {
    Ok(value.clone().unbind());
    Ok(py.None())
}

fn dig_args(py: Python, obj: &Bound<'_, PyAny>, args: Bound<'_, PyTuple>) -> PyResult<PyObject> {
    Ok(obj.into_py(py))
}

// #[pyclass]
// struct DynAttr {
//     path: Vec<PyObject>,
// }
//
// #[pymethods]
// impl DynAttr {
//     #[new]
//     fn new() -> Self {
//         DynAttr { path: Vec::new() }
//     }
//
//     fn __getitem__(&mut self, key: PyObject, py: Python) -> PyResult<&DynAttr> {
//         self.path.push(key.clone_ref(py));
//         Ok(self)
//     }
//
//     fn __getattr__(&mut self, name: PyObject, py: Python) -> PyResult<&DynAttr> {
//         self.path.push(name.clone_ref(py));
//         Ok(self)
//     }
//
//     #[pyo3(signature = (obj, r#default=None))]
//     fn __call__(
//         &self,
//         py: Python,
//         obj: PyObject,
//         r#default: Option<&Bound<'_, PyAny>>,
//     ) -> PyResult<PyObject> {
//         let mut current = obj.to_object(py);
//
//         for part in &self.path {
//             current = match current {
//                 obj if obj.is_instance::<PyDict>(py) => {
//                     let dict = obj.downcast::<PyDict>(py)?;
//                     dict.get_item(part)
//                         .map_err(|_| py.new_err(format!("Key '{}' not found", part)))?
//                         .to_object(py)
//                 }
//                 obj if obj.is_instance::<PyList>(py) => {
//                     let list = obj.downcast::<PyList>(py)?;
//                     let index = part
//                         .parse::<usize>()
//                         .map_err(|_| py.new_err(format!("Invalid index '{}'", part)))?;
//                     list.get_item(index)
//                         .map_err(|_| py.new_err(format!("Index '{}' out of range", part)))?
//                         .to_object(py)
//                 }
//                 _ => {
//                     return Err(py.new_err(format!(
//                         "Invalid type '{}' for path segment '{}'",
//                         current.get_type().name(),
//                         part
//                     )))
//                 }
//             }
//         }
//
//         Ok(current)
//     }
// }
//
// #[pymodule]
// fn mylib(_py: Python, m: &PyModule) -> PyResult<()> {
//     m.add_class::<DynAttr>()?;
//     Ok(())
// }
//
// fn main() {
//     Python::with_gil(|py| {
//         let module = PyModule::new(py, "mylib").unwrap();
//         mylib(py, &module).unwrap();
//         module
//             .add_submodule("dyn_attr", PyModule::new(py, "dyn_attr").unwrap())
//             .unwrap();
//     });
// }

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
