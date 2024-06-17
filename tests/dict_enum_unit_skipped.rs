use std::collections::HashMap;

use pyo3::{pyclass, types::PyAnyMethods, Python};
use pyo3_special_method_derive::Dict;

#[pyclass]
#[derive(Dict)]
#[allow(dead_code)]
enum Tester {
    Alpha,
    #[skip]
    Beta,
}

#[test]
fn test_dict() {
    pyo3::prepare_freethreaded_python();

    Python::with_gil(|py| {
        let res = pyo3::Py::new(py, Tester::Beta).unwrap();

        let dict = {
            let py_any_ref = res.bind(py).as_any().getattr("__dict__").unwrap();
            py_any_ref
                .extract::<HashMap<String, pyo3::Py<pyo3::PyAny>>>()
                .unwrap()
        };

        assert!(dict.is_empty())
    });
}
