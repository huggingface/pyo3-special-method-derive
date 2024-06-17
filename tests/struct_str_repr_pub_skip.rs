use pyo3::pyclass;
use pyo3_special_method_derive::{Dir, Repr, Str};

#[pyclass]
#[derive(Dir, Str, Repr)]
#[allow(dead_code)]
struct WithFields {
    pub dora: u32,
    my: String,
    #[skip]
    pub name: f32,
}

#[test]
fn test_with_str() {
    let res = WithFields {
        dora: 299792458,
        my: "Hello world".to_string(),
        name: std::f32::consts::PI,
    }
    .__str__();
    assert_eq!("WithFields(dora=299792458)", &res);
}

#[test]
fn test_with_repr() {
    let res = WithFields {
        dora: 299792458,
        my: "Hello world".to_string(),
        name: std::f32::consts::PI,
    }
    .__repr__();
    assert_eq!("WithFields(dora=299792458)", &res);
}
