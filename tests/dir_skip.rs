use pyo3::pyclass;
use pyo3_special_method_derive::DirHelper;

#[pyclass]
#[derive(DirHelper)]
#[allow(dead_code)]
struct WithFieldSkip {
    dora: u32,
    my: String,
    #[skip]
    name: f32,
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
#[derive(DirHelper)]
#[allow(dead_code)]
struct WithAllFieldsSkiped {
    #[skip]
    dora: u32,
    #[skip]
    my: String,
    #[skip]
    name: f32,
}

#[test]
fn test_with_dir_all_skipped() {
    let dir = WithAllFieldsSkiped {
        dora: 0,
        my: "".to_string(),
        name: 0.0,
    }
    .__dir__();
    assert!(dir.is_empty());
}
