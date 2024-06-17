use pyo3::{pyclass, types::PyAnyMethods, Python};
use pyo3_special_method_derive::Dict;

#[pyclass]
#[derive(Dict)]
#[allow(dead_code)]
struct Person {
    pub name: String,
    pub address: String,
    location: String,
}

fn main() {
    pyo3::prepare_freethreaded_python();

    let res = Person {
        name: "John Doe".to_string(),
        address: "Address".to_string(),
        location: "Earth".to_string(),
    }
    .__dict__();

    let mut keys = res.keys().cloned().collect::<Vec<_>>();
    keys.sort();
    let mut values = Vec::new();
    for k in &keys {
        let v = res.get(k).unwrap();
        values.push(Python::with_gil(|py| {
            let py_any_ref = v.bind(py);
            py_any_ref.extract::<String>().unwrap()
        }));
    }
    println!("Keys: {keys:?} Values {values:?}");
}
