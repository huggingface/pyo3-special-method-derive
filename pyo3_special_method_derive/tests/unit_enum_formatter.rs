use pyo3::pyclass;
use pyo3_special_method_derive::Str;

#[pyclass(eq, eq_int)]
#[derive(Str, PartialEq)]
#[format(fmt = "Enum: {}.{}")]
enum Data {
    Alpha,
}

#[test]
fn test_formatter_enum() {
    let data = Data::Alpha;

    assert_eq!(data.__str__(), "Enum: Data.Alpha");
}
