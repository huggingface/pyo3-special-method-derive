use std::collections::HashMap;

use pyo3::{pyclass, types::PyAnyMethods, Python};
use pyo3_special_method_derive::DictHelper;

#[pyclass]
#[derive(DictHelper)]
#[allow(dead_code)]
enum Tester {
    Alpha { x: String },
    Beta { x: String, y: String },
}

#[test]
fn test_dict() {
    pyo3::prepare_freethreaded_python();

    Python::with_gil(|py| {
        let res = pyo3::Py::new(
            py,
            Tester::Beta {
                x: "What is 1+2".to_string(),
                y: "Hello world".to_string(),
            },
        )
        .unwrap();

        let dict = {
            let py_any_ref = res.bind(py).as_any().getattr("__dict__").unwrap();
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
        assert_eq!(keys, vec!["x".to_string(), "y".to_string()]);
        assert_eq!(
            values,
            vec!["What is 1+2".to_string(), "Hello world".to_string()]
        );
    });
}
