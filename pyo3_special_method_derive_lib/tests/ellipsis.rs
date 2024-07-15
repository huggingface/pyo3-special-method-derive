use std::collections::HashMap;

use pyo3_special_method_derive_lib::PyDisplay;

#[test]
fn vec() {
    assert_eq!(
        (0..150).collect::<Vec<_>>().fmt_display(),
        "[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 95, 96, 97, 98, 99, ...]"
    );
}

#[test]
fn hashmap() {
    let mut map = HashMap::new();
    for i in 0..150 {
        map.insert(format!("{i:0w$}", w = 5), format!("{i:0w$}", w = 5));
    }
    // parens + ellipsis len * (2 quotes + 5 chars) * 2 kv + 100 ", " + 100 ": " + "..."
    assert_eq!(
        map.fmt_display().len(),
        2 + 100 * (7 * 2) + 2 * 100 + 2 * 100 + 3
    );
}
