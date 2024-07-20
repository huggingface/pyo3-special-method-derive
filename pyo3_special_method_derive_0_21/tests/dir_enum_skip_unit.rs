use pyo3::pyclass;
use pyo3_special_method_derive_0_21::Dir;

#[derive(PartialEq)]
#[pyclass]
#[derive(Dir)]
#[allow(dead_code)]
enum Tester {
    Alpha,
    #[pyo3_smd(skip)]
    Beta,
}

#[test]
fn test_with_dir() {
    let dir = Tester::Beta.__dir__();

    assert!(dir.is_empty());
}
