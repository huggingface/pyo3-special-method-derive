use pyo3::pyclass;
use pyo3_special_method_derive_0_21::{Repr, Str};

#[derive(PartialEq)]
#[pyclass]
#[derive(Str, Repr)]
#[allow(dead_code)]
enum Tester {
    Alpha,
    #[pyo3_smd_repr(skip)]
    Beta,
    #[pyo3_smd_str(skip)]
    Gamma,
}

#[test]
fn test_with_str_skip() {
    let res = Tester::Beta.__repr__();
    assert_eq!("Tester.<variant skipped>", &res);
}

#[test]
fn test_with_repr_skip() {
    let res = Tester::Gamma.__str__();
    assert_eq!("Tester.<variant skipped>", &res);
}

#[test]
fn test_with_str_repr() {
    let res = Tester::Alpha.__str__();
    assert_eq!("Tester.Alpha", &res);
}
