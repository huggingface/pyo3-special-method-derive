use pyo3::{pyclass, pymethods, pymodule, types::PyModule, PyResult, Python};
use pyo3_special_method_derive::{AutoDebug, AutoDisplay, Dict, Dir, Getattr, Repr, Str};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, RwLock};

#[derive(Clone, AutoDisplay, AutoDebug, PartialEq, Eq, Hash, Default)]
#[auto_display(fmt = "City.{}")]
pub enum CityName {
    Paris,
    #[default]
    London,
    #[auto_display(fmt = "CityName:{}")]
    NewYork,
}

impl FromStr for CityName {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "paris" => Ok(CityName::Paris),
            "london" => Ok(CityName::London),
            "new york" => Ok(CityName::NewYork),
            _ => Err(()),
        }
    }
}

#[derive(Default, AutoDebug, AutoDisplay)]
pub struct City {
    name: CityName,
    addresses: HashMap<String, Arc<RwLock<PyAddress>>>,
}

impl City {
    pub fn new(name: CityName) -> Self {
        City {
            name,
            addresses: HashMap::new(),
        }
    }

    pub fn is_address_occupied(&self, address_key: &str) -> bool {
        self.addresses.contains_key(address_key)
    }

    pub fn occupy_address(&mut self, address_key: String, address: Arc<RwLock<PyAddress>>) {
        self.addresses.insert(address_key, address);
    }

    pub fn free_address(&mut self, address_key: &str) {
        self.addresses.remove(address_key);
    }
}

#[derive(Default, AutoDebug, AutoDisplay)]
pub struct Country {
    capital: Option<CityName>,
    pub regions: Vec<CityName>,
    pub(crate) hash: Option<u128>,
    inhabitants: HashMap<CityName, u32>, // it's private but I still want to be able to display it!
}

#[derive(AutoDisplay, AutoDebug)]
pub struct Region;

#[derive(AutoDisplay, AutoDebug)]
pub enum MyObjectWrapper {
    Country(Country),
    City(CityName),
    CountryRegion(Region),
}

#[pyclass]
#[derive(Repr, Str, Clone)]
pub struct PyCity {
    pub(crate) city: Arc<RwLock<City>>,
}

#[pymethods]
impl PyCity {
    #[new]
    pub fn new(name: String) -> PyResult<Self> {
        match CityName::from_str(&name) {
            Ok(city_name) => Ok(PyCity {
                city: Arc::new(RwLock::new(City::new(city_name))),
            }),
            Err(_) => Err(pyo3::exceptions::PyValueError::new_err("Invalid city name")),
        }
    }

    pub fn is_address_occupied(&self, address_key: String) -> bool {
        let city = self.city.read().unwrap();
        city.is_address_occupied(&address_key)
    }
}

#[pyclass]
#[derive(Dir, Dict, Str, Repr, Getattr, Clone)]
pub enum PyAddress {
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
    #[pyo3_no_skip]
    address: Arc<RwLock<PyAddress>>,
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
    ) -> Self {
        let address = PyAddress::House {
            country,
            city: city.clone(),
            street: street.clone(),
            street_number,
        };
        let address_arc = Arc::new(RwLock::new(address));
        {
            let mut city = city.city.write().unwrap();
            let address_key = format!("{}-{}", street, street_number);
            city.occupy_address(address_key, address_arc.clone());
        }
        Self {
            name,
            age,
            address: address_arc,
        }
    }

    pub fn change_address(
        &mut self,
        new_country: String,
        new_city: PyCity,
        new_street: String,
        new_street_number: u32,
    ) {
        let new_address_key = format!("{}-{}", new_street, new_street_number);
        {
            let adress = self.address.read().unwrap();
            let mut current_city = adress.get_city().city.write().unwrap();
            let current_address_key = self.address.read().unwrap().get_address_key();
            current_city.free_address(&current_address_key);
        }
        let new_address = PyAddress::House {
            country: new_country,
            city: new_city.clone(),
            street: new_street,
            street_number: new_street_number,
        };
        let new_address_arc = Arc::new(RwLock::new(new_address));
        {
            let mut city = new_city.city.write().unwrap();
            city.occupy_address(new_address_key, Arc::clone(&new_address_arc));
        }
        self.address = new_address_arc;
    }

    pub fn get_address(&self) -> String {
        let address = self.address.read().unwrap();
        address.get_full_address()
    }
}

impl PyAddress {
    fn get_city(&self) -> &PyCity {
        match self {
            PyAddress::House { city, .. } => city,
        }
    }

    fn get_address_key(&self) -> String {
        match self {
            PyAddress::House {
                street,
                street_number,
                ..
            } => format!("{}-{}", street, street_number),
        }
    }

    fn get_full_address(&self) -> String {
        match self {
            PyAddress::House {
                country,
                city,
                street,
                street_number,
            } => {
                format!(
                    "{}, {}, {}-{}",
                    country,
                    city.city.read().unwrap().name,
                    street,
                    street_number
                )
            }
        }
    }
}

#[pymodule]
fn pyo3_smd_example(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Person>()?;
    m.add_class::<PyCity>()?;
    Ok(())
}
