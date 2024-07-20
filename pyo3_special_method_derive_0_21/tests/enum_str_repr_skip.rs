use pyo3::pyclass;
use pyo3_special_method_derive_0_21::{Repr, Str};

#[derive(PartialEq)]
#[pyclass]
#[derive(Str, Repr)]
#[allow(dead_code)]
enum Tester {
    Alpha,
    Beta,
    #[pyo3_smd(skip)]
    Gamma,
}

#[test]
fn test_with_str() {
    let res = Tester::Alpha.__str__();
    assert_eq!("Tester.Alpha", &res);
}

#[test]
fn test_with_repr() {
    let res = Tester::Gamma.__repr__();
    assert_eq!("Tester.<variant skipped>", &res);
}
