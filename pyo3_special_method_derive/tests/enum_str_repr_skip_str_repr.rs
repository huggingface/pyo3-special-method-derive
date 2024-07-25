use pyo3::pyclass;
use pyo3_special_method_derive::{Repr, Str};

#[derive(PartialEq)]
#[pyclass(eq, eq_int)]
#[derive(Str, Repr)]
#[allow(dead_code)]
enum Tester {
    Alpha,
    #[skip(Str, Repr)]
    Beta,
    #[skip(Str)]
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
