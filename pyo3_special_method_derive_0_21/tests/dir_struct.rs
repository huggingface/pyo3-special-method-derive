use pyo3::pyclass;
use pyo3_special_method_derive_0_21::Dir;

#[pyclass]
#[derive(Dir)]
#[allow(dead_code)]
struct Person {
    pub name: String,
    pub address: String,
    location: String,
}

#[test]
fn test_with_dir_skip() {
    let dir = Person {
        name: "John Doe".to_string(),
        address: "Address".to_string(),
        location: "Earth".to_string(),
    }
    .__dir__();

    assert_eq!(dir, vec!["name".to_string(), "address".to_string(),]);
}
