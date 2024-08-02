use pyo3::pyclass;
use pyo3_special_method_derive_0_21::Dir;

#[pyclass]
#[derive(Dir)]
enum X {
    A {},
}
