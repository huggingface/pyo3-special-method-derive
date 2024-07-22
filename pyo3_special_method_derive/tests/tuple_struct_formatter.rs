use pyo3::pyclass;
use pyo3_special_method_derive::Str;

#[pyclass]
#[derive(Str)]
#[formatter(fmt = "Struct: {}({})")]
struct Data(
    #[formatter(fmt = "{}")] pub usize,
    #[formatter(fmt = "[{}]")] pub f32,
);

#[test]
fn test_formatter_struct() {
    let data = Data(5, 1.23);

    assert_eq!(data.__str__(), "Struct: Data(0=5, 1=[1.23])");
}
