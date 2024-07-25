use pyo3::pyclass;
use pyo3_special_method_derive_0_21::Str;

#[pyclass]
#[derive(Str)]
#[format(fmt = "Struct: {}({})")]
struct Data(
    #[format(fmt = "{}")] pub usize,
    #[format(fmt = "[{}]")] pub f32,
);

#[test]
fn test_formatter_struct() {
    let data = Data(5, 1.23);

    assert_eq!(data.__str__(), "Struct: Data(0=5, 1=[1.23])");
}
