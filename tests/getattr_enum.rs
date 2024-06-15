use pyo3::{pyclass, types::PyAnyMethods, Python};
use pyo3_special_method_derive::GetattrHelper;

#[pyclass]
#[derive(GetattrHelper)]
enum Tester {
    Alpha { x: String },
    Beta { x: String, y: String },
}

#[test]
fn test_get() {
    pyo3::prepare_freethreaded_python();

    let res = Tester::Beta {
        x: "What is 1+2?".to_string(),
        y: "Hello world".to_string(),
    }
    .__getattr__("x".to_string())
    .unwrap();

    let name = Python::with_gil(|py| {
        let py_any_ref = res.bind(py);
        py_any_ref.extract::<String>().unwrap()
    });
    assert_eq!(&name, "What is 1+2?");
}

#[test]
fn test_get_attr_exception() {
    pyo3::prepare_freethreaded_python();

    let res = Tester::Beta {
        x: "What is 1+2?".to_string(),
        y: "Hello world".to_string(),
    }
    .__getattr__("z".to_string())
    .unwrap_err();

    let correct_err = Python::with_gil(|py| {
        &res.value_bound(py).to_string() == "'Tester.Beta' has no attribute 'z'"
    });
    assert!(correct_err);
}
