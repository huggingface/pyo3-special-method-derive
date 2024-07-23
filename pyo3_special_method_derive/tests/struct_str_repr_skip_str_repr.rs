use pyo3::pyclass;
use pyo3_special_method_derive::{Dir, Repr, Str};

#[pyclass]
#[derive(Dir, Str, Repr)]
#[allow(dead_code)]
struct WithFields {
    pub dora: u32,
    #[skip(Repr)]
    pub my: String,
    #[skip(Str)]
    pub name: f32,
}

#[test]
fn test_with_str_skip() {
    let res = WithFields {
        dora: 299792458,
        my: "Hello world".to_string(),
        name: std::f32::consts::PI,
    }
    .__str__();
    assert_eq!("WithFields(dora=299792458, my=\"Hello world\")", &res);
}

#[test]
fn test_with_repr_skip() {
    let res = WithFields {
        dora: 299792458,
        my: "Hello world".to_string(),
        name: std::f32::consts::PI,
    }
    .__repr__();
    assert_eq!(
        format!("WithFields(dora=299792458, name={})", std::f32::consts::PI),
        res
    );
}
