use pyo3::pyclass;
use pyo3_special_method_derive::Dir;

#[pyclass]
#[derive(Dir)]
#[allow(dead_code)]
enum Tester {
    Alpha { x: String },
    Beta { x: String, y: String },
}

#[test]
fn test_with_dir() {
    let dir = Tester::Beta {
        x: "Hello".to_string(),
        y: "World".to_string(),
    }
    .__dir__();

    assert_eq!(dir, vec!["x".to_string(), "y".to_string()]);
}
