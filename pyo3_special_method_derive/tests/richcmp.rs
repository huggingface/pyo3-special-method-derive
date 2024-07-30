use pyo3::{basic::CompareOp, pyclass};
use pyo3_special_method_derive::richcmp_derive_with;

#[derive(PartialEq, PartialOrd)]
#[pyclass]
#[richcmp_derive_with(PartialEq, PartialOrd)]
struct Point(f32, f32);

#[test]
fn eq() {
    let a = Point(1., 1.);
    let b = Point(1., 1.);
    assert!(a.__richcmp__(&b, CompareOp::Eq).unwrap());
}

#[test]
fn le() {
    let a = Point(2., 2.);
    let b = Point(1., 1.);
    assert!(a.__richcmp__(&b, CompareOp::Gt).unwrap());
}
