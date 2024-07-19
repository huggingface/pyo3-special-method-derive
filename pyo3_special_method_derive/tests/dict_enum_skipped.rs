use std::collections::HashMap;

use pyo3::{pyclass, types::PyAnyMethods, Python};
use pyo3_special_method_derive::Dict;

#[pyclass]
#[derive(Dict)]
#[allow(dead_code)]
enum Tester {
    Alpha {
        x: String,
    },
    #[pyo3_smd(skip)]
    Beta {
        x: String,
        y: String,
    },
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

        assert!(dict.is_empty())
    });
}
