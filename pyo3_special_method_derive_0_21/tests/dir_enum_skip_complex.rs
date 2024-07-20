use pyo3::pyclass;
use pyo3_special_method_derive_0_21::Dir;

#[pyclass]
#[derive(Dir)]
#[allow(dead_code)]
enum Tester {
    Alpha {
        x: String,
    },
    #[pyo3_smd(skip)]
    Beta {
        x: String,
        y: String,
    },
}

#[test]
fn test_with_dir() {
    let dir = Tester::Beta {
        x: "Hello".to_string(),
        y: "World".to_string(),
    }
    .__dir__();

    assert!(dir.is_empty());
}
