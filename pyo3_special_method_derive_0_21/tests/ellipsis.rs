use std::collections::HashMap;

use pyo3_special_method_derive_0_21::PyDisplay;

#[test]
fn vec() {
    assert_eq!((0..150).collect::<Vec<_>>().fmt_display().len(), 103,);
}

#[test]
fn hashmap() {
    let mut map = HashMap::new();
    for i in 0..150 {
        map.insert(format!("{i:0w$}", w = 5), format!("{i:0w$}", w = 5));
    }
    assert_eq!(map.fmt_display().len(), 95,);
}
