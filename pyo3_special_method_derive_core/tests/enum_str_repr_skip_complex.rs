use pyo3::pyclass;
use pyo3_special_method_derive::{Repr, Str};

#[pyclass]
#[derive(Str, Repr)]
#[allow(dead_code)]
enum Tester {
    Alpha {
        x: String,
    },
    Beta {
        x: u32,
        y: u32,
    },
    #[pyo3_smd(skip)]
    Gamma {
        x: u32,
        y: u32,
        z: u32,
    },
}

#[test]
fn test_with_str() {
    let res = Tester::Alpha {
        x: "Hello!".to_string(),
    }
    .__str__();
    assert_eq!("Tester.Alpha(x=\"Hello!\")", &res);
}

#[test]
fn test_with_repr() {
    let res = Tester::Gamma { x: 1, y: 2, z: 3 }.__repr__();
    assert_eq!("Tester.<variant skipped>", &res);
}
