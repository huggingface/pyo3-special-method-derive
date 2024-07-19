use pyo3::pyclass;
use pyo3_special_method_derive::Dir;

#[pyclass]
#[derive(Dir)]
enum X {
    A {},
}
