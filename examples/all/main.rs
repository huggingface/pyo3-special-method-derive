use pyo3::pyclass;
use pyo3_special_method_derive::{Dir, StrRepr};

#[pyclass]
#[derive(StrRepr, Dir)]
enum Tester {
    Alpha {
        x: u32,
    },
    Beta {
        x: u32,
        y: u32,
    },
    #[skip]
    Gamma {
        x: u32,
        y: u32,
        z: u32,
    },
}

#[pyclass]
#[derive(Dir, StrRepr)]
#[allow(dead_code)]
struct Person {
    pub name: String,
    occupation: String,
    #[skip]
    pub phone_num: String,
}

fn main() {
    let person = Person {
        name: "John Doe".to_string(),
        occupation: "Programmer".to_string(),
        phone_num: "123 456 7890".to_string(),
    };
    assert_eq!(person.__dir__(), vec!["name".to_string()]);
    assert_eq!(person.__str__(), "Person(name=\"John Doe\")");

    let tester_beta = Tester::Beta { x: 123, y: 456 };
    assert_eq!(
        tester_beta.__dir__(),
        vec!["Alpha".to_string(), "Beta".to_string()]
    );
    assert_eq!(tester_beta.__str__(), "Tester.Beta(x=123, y=456)");

    let tester_gamma = Tester::Gamma {
        x: 123,
        y: 456,
        z: 789,
    };
    assert_eq!(
        tester_gamma.__dir__(),
        vec!["Alpha".to_string(), "Beta".to_string()]
    );
    assert_eq!(tester_gamma.__str__(), "<variant skipped>");
}
