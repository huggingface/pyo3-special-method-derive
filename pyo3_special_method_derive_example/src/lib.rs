use pyo3::{pyclass, pymethods, pymodule, types::PyModule, Bound, PyResult, Python};
use pyo3_special_method_derive::{Dict, Dir, Getattr, Repr, Str};

#[pyclass]
#[derive(Dir, Dict, Str, Repr, Getattr, Clone)]
pub enum Address {
    House {
        country: String,
        city: String,
        street: String,
        street_number: u32,
    },
    Apartment {
        country: String,
        city: String,
        street: String,
        street_number: u32,
        number: u32,
    },
}

#[pyclass]
#[derive(Dir, Dict, Str, Repr, Getattr)]
pub struct Person {
    pub name: String,
    pub age: u8,
    pub address: Address,
}

#[pymethods]
impl Person {
    #[new]
    pub fn new_dummy() -> Self {
        Self {
            name: "Name here".to_string(),
            age: 0,
            address: Address::House {
                country: "Country here".to_string(),
                city: "City here".to_string(),
                street: "Street here".to_string(),
                street_number: u32::MAX,
            },
        }
    }
}

#[pymodule]
fn pyo3_smd_example(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Person>()?;
    Ok(())
}
