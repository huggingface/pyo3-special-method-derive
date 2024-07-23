use pyo3::pyclass;
use pyo3_special_method_derive::Dir;

#[pyclass]
#[derive(Dir)]
#[allow(dead_code)]
struct WithFieldSkip {
    pub dora: u32,
    pub my: String,
    #[skip(Dir)]
    pub name: f32,
}

#[test]
fn test_with_dir_skip() {
    let dir = WithFieldSkip {
        dora: 0,
        my: "".to_string(),
        name: 0.0,
    }
    .__dir__();
    assert_eq!(vec!["dora".to_string(), "my".to_string()], dir);
}

#[pyclass]
#[derive(Dir)]
#[allow(dead_code)]
struct WithAllFieldsSkipped {
    #[skip(Dir)]
    pub dora: u32,
    #[skip(Dir)]
    pub my: String,
    #[skip(Dir)]
    pub name: f32,
}

#[test]
fn test_with_dir_all_skipped() {
    let dir = WithAllFieldsSkipped {
        dora: 0,
        my: "".to_string(),
        name: 0.0,
    }
    .__dir__();
    assert!(dir.is_empty());
}
