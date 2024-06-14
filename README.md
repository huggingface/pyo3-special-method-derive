# pyo3-special-method-derive

This crate enables you to automatically derive Python dunder methods for your Rust crate using PyO3.

## Key features
- The following methods may be automatically derived
    - `__str__`
    - `__repr__`
    - `__dir__`
- Support for structs and enums (only unit and complex enums due to a PyO3 limitation)
- Support for skipping variants or fields with the `#[skip]` attribute
- Automatically skip struct fields which are not `pub`.

> Note: The `StrReprHelper` macro requires `T: Debug` for each `T` inside the item. The `Debug` trait is used for the outputs.

Coming soon:
- `__dict__`
- Automatic derive of `Debug` for broader `StrReprHelper` support
- Skip different fields/variants depending on `__str__` or `__repr__`.

## Example
```rust
#[pyclass]
#[derive(DirHelper, StrReprHelper)]
struct Person {
    pub name: String,
    occupation: String,
    #[skip]
    pub phone_num: String,
}
```

## PyO3 feature note
To use `pyo3-special-method-derive`, you should enable the `multiple-pymethods` feature on PyO3:
```
pyo3 = { version = "0.21", features = ["multiple-pymethods"] }
```