use pyo3::{pyclass, Python};
use pyo3_special_method_derive::Getattr;

#[derive(PartialEq)]
#[pyclass(eq, eq_int)]
#[derive(Getattr)]
enum Tester {
    Alpha,
    #[skip(Getattr)]
    Beta,
}

#[test]
fn test_get_attr_exception() {
    pyo3::prepare_freethreaded_python();

    let res = Tester::Beta.__getattr__("z".to_string()).unwrap_err();

    let correct_err = Python::with_gil(|py| {
        &res.value_bound(py).to_string() == "'Tester.Beta' has no attribute 'z'"
    });
    assert!(correct_err);
}
