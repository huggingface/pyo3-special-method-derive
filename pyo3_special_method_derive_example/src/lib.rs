use pyo3::{pyclass, pymethods, pymodule, types::PyModule, Bound, PyResult, Python};
use pyo3_special_method_derive::{Dict, Dir, Getattr, Repr, Str};
use pyo3_special_method_derive::{AutoDebug, AutoDisplay};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
#[derive(Default, Clone, AutoDisplay, AutoDebug)]
pub enum City {
    Paris, 
    #[default]
    London
}

#[derive(Default, AutoDebug, AutoDisplay)]
pub struct Country {
    capital : Option<City>,   
    pub regions: Vec<City> ,
    pub(crate) hash: Option<u128>,
    inhabitants: HashMap<City, u32>, // it's private but I still want to be able to display it! 
}

#[derive(AutoDisplay, AutoDebug)]
pub struct Region;

// I want the display to be "MyObject." not MyObjectWrapper
#[derive(AutoDisplay, AutoDebug)]
pub enum MyObjectWrapper {
    Country(Country), 
    City(City),
    CountryRegion(Region)
}


#[pyclass]
#[derive(Repr, Str, Clone)]
pub struct PyCity{
    pub(crate) city: City,
}

#[pyclass]
#[derive(Dir, Dict, Str, Repr, Getattr, Clone)]
pub enum Address {
    House {
        country: String,
        city: PyCity,
        street: String,
        street_number: u32,
    },
}

#[pyclass]
#[derive(Dir, Str, Repr, Getattr, Dict)]
pub struct Person {
    pub name: String,
    pub age: u8,
    pub address: Address,
   //  #[auto_display(fmt = "{}", "_0.as_ref().read().unwrap()")]
    houses: Arc<RwLock<Address>>, // I need to be able to do as_ref().read().unwrap()
}

#[pymethods]
impl Person {
    #[new]
    pub fn new(
        name: String,
        age: u8,
        country: String,
        city: PyCity,
        street: String,
        street_number: u32,
        houses: Address, 
    ) -> Self {
        Self {
            name,
            age,
            address: Address::House {
                country,
                city,
                street,
                street_number,
            },
            houses: Arc::new(RwLock::new(houses)),
        }
    }
}

#[pymodule]
fn pyo3_smd_example(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Person>()?;
    Ok(())
}
