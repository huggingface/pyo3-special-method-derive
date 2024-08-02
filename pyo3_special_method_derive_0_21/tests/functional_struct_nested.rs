use pyo3::pyclass;
use pyo3_special_method_derive_0_21::{Dir, Repr, Str};

#[pyclass]
#[derive(Dir, Str, Repr)]
#[allow(dead_code)]
struct B {
    pub z: u32,
}

#[pyclass]
#[derive(Dir, Str, Repr)]
#[allow(dead_code)]
struct A {
    pub x: u32,
    pub y: Option<B>,
}
