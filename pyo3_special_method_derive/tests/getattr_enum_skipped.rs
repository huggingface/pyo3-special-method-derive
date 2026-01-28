use pyo3::{pyclass, Python};
use pyo3_special_method_derive::Getattr;

#[pyclass]
#[derive(Getattr)]
enum Tester {
    Alpha {
        x: String,
    },
    #[skip(Getattr)]
    Beta {
        x: String,
        y: String,
    },
}

#[test]
fn test_get_attr_exception() {
    Python::initialize();

    let res = Tester::Beta {
        x: "What is 1+2?".to_string(),
        y: "Hello world".to_string(),
    }
    .__getattr__("z".to_string())
    .unwrap_err();

    let correct_err =
        Python::attach(|py| &res.value(py).to_string() == "'Tester.Beta' has no attribute 'z'");
    assert!(correct_err);
}
