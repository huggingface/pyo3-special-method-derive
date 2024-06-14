use pyo3::{pyclass, types::PyAnyMethods, Python};
use pyo3_special_method_derive::GetattrHelper;

#[pyclass]
#[derive(GetattrHelper)]
struct Person {
    pub name: String,
    pub address: String,
}

#[test]
fn test_get() {
    pyo3::prepare_freethreaded_python();

    let res = Person {
        name: "John Doe".to_string(),
        address: "Address".to_string(),
    }
    .__getattr__("name".to_string())
    .unwrap();

    let name = Python::with_gil(|py| {
        let py_any_ref = res.bind(py);
        py_any_ref.extract::<String>().unwrap()
    });
    assert_eq!(&name, "John Doe");
}

#[test]
fn test_get_attr_exception() {
    pyo3::prepare_freethreaded_python();

    let res = Person {
        name: "John Doe".to_string(),
        address: "Address".to_string(),
    }
    .__getattr__("not_name".to_string())
    .unwrap_err();

    let correct_err = Python::with_gil(|py| {
        &res.value_bound(py).to_string() == "'Person' has no attribute 'not_name'"
    });
    assert!(correct_err);
}
