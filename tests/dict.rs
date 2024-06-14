use pyo3::{pyclass, types::PyAnyMethods, Python};
use pyo3_special_method_derive::DictHelper;

#[pyclass]
#[derive(DictHelper)]
#[allow(dead_code)]
struct Person {
    pub name: String,
    pub address: String,
    location: String,
}

#[test]
fn test_dict() {
    pyo3::prepare_freethreaded_python();

    let res = Person {
        name: "John Doe".to_string(),
        address: "Address".to_string(),
        location: "Earth".to_string(),
    }
    .__dict__();

    let mut keys = res.keys().into_iter().cloned().collect::<Vec<_>>();
    keys.sort();
    let mut values = Vec::new();
    for k in &keys {
        let v = res.get(k).unwrap();
        values.push(Python::with_gil(|py| {
            let py_any_ref = v.bind(py);
            py_any_ref.extract::<String>().unwrap()
        }));
    }
    assert_eq!(keys, vec!["address".to_string(), "name".to_string()]);
    assert_eq!(values, vec!["Address".to_string(), "John Doe".to_string()]);
}
