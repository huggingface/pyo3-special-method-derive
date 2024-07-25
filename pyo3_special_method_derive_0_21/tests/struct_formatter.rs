use pyo3::pyclass;
use pyo3_special_method_derive_0_21::Str;

#[pyclass]
#[derive(Str)]
#[format(fmt = "Struct: {}({})")]
struct Data {
    #[format(fmt = "{}")]
    pub x: usize,
    #[format(fmt = "[{}]")]
    pub y: f32,
}

#[test]
fn test_formatter_struct() {
    let data = Data { x: 5, y: 1.23 };

    assert_eq!(data.__str__(), "Struct: Data(x=5, y=[1.23])");
}
