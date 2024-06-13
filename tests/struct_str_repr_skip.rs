use pyo3::pyclass;
use pyo3_special_method_derive::{DirHelper, StrReprHelper};

#[pyclass]
#[derive(DirHelper, StrReprHelper)]
#[allow(dead_code)]
struct WithFields {
    dora: u32,
    my: String,
    #[skip]
    name: f32,
}

#[test]
fn test_with_str() {
    let res = WithFields {
        dora: 299792458,
        my: "Hello world".to_string(),
        name: 3.14159,
    }
    .__str__();
    assert_eq!("WithFields(dora=299792458, my=\"Hello world\")", &res);
}

#[test]
fn test_with_repr() {
    let res = WithFields {
        dora: 299792458,
        my: "Hello world".to_string(),
        name: 3.14159,
    }
    .__repr__();
    assert_eq!("WithFields(dora=299792458, my=\"Hello world\")", &res);
}
