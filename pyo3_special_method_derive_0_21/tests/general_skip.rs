use pyo3::pyclass;
use pyo3_special_method_derive_0_21::Dir;

#[allow(dead_code)]
#[pyclass]
#[derive(Dir)]
struct Data {
    #[skip(Dir)]
    pub x: usize,
    pub y: f32,
}

#[test]
fn test_with_dir_skip() {
    let dir = Data { x: 5, y: 1.23 }.__dir__();

    assert_eq!(dir, vec!["y".to_string(),]);
}
