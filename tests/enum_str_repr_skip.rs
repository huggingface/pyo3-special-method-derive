use pyo3::pyclass;
use pyo3_special_method_derive::StrRepr;

#[pyclass]
#[derive(StrRepr)]
#[allow(dead_code)]
enum Tester {
    Alpha,
    Beta,
    #[skip]
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
    assert_eq!("<variant skipped>", &res);
}
