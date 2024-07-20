use pyo3::pyclass;
use pyo3_special_method_derive_0_21::Getattr;

#[pyclass]
#[derive(Getattr)]
#[allow(dead_code)]
struct Person {
    pub name: String,
    pub occupation: String,
    phone_num: String,
}

fn main() {
    pyo3::prepare_freethreaded_python();

    let person = Person {
        name: "John Doe".to_string(),
        occupation: "Programmer".to_string(),
        phone_num: "123 456 7890".to_string(),
    };

    println!("{:?}", person.__getattr__("name".to_string()).unwrap());
    println!(
        "{:?}",
        person.__getattr__("phone_num".to_string()).unwrap_err()
    );
}
