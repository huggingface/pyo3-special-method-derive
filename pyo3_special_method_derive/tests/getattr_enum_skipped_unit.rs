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
    Python::initialize();

    let res = Tester::Beta.__getattr__("z".to_string()).unwrap_err();

    let correct_err =
        Python::attach(|py| &res.value(py).to_string() == "'Tester.Beta' has no attribute 'z'");
    assert!(correct_err);
}
