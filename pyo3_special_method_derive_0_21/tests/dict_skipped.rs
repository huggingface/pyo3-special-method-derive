use std::collections::HashMap;

use pyo3::{pyclass, types::PyAnyMethods, Python};
use pyo3_special_method_derive_0_21::Dict;

#[pyclass]
#[derive(Dict)]
#[allow(dead_code)]
struct Person {
    pub name: String,
    #[skip(Dict)]
    pub address: String,
    location: String,
}

#[test]
fn test_dict() {
    pyo3::prepare_freethreaded_python();

    Python::with_gil(|py| {
        let res = pyo3::Py::new(
            py,
            Person {
                name: "John Doe".to_string(),
                address: "Address".to_string(),
                location: "Earth".to_string(),
            },
        )
        .unwrap();

        let dict = {
            let py_any_ref = res.bind(py).getattr("__dict__").unwrap();
            py_any_ref
                .extract::<HashMap<String, pyo3::Py<pyo3::PyAny>>>()
                .unwrap()
        };

        let mut keys = dict.keys().cloned().collect::<Vec<_>>();
        keys.sort();
        let mut values = Vec::new();
        for k in &keys {
            let v = dict.get(k).unwrap().bind(py);
            values.push(v.extract::<String>().unwrap());
        }
        assert_eq!(keys, vec!["name".to_string()]);
        assert_eq!(values, vec!["John Doe".to_string()]);
    });
}
