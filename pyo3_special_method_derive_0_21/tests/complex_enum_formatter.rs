use pyo3::pyclass;
use pyo3_special_method_derive::Str;

#[pyclass]
#[derive(Str)]
#[format(fmt = "Enum: {}.{}")]
enum Data {
    Alpha {
        #[format(fmt = "A[{}]")]
        x: usize,
        #[format(fmt = "B[{}]")]
        y: f32,
    },
}

#[test]
fn test_formatter_enum() {
    let data = Data::Alpha { x: 5, y: 1.23 };

    assert_eq!(data.__str__(), "Enum: Data.Alpha(x=A[5], y=B[1.23])");
}
