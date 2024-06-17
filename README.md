# pyo3-special-method-derive

This crate enables you to automatically derive Python dunder methods for your Rust crate using PyO3.

## Key features
- The following methods may be automatically derived on structs and enums:
    - `__str__`
    - `__repr__`
    - `__dir__`
    - `__getattr__`
    - `__dict__`
- Support for structs and enums (only unit and complex enums due to a PyO3 limitation)
- Support for skipping variants or fields with the `#[skip]` attribute
- Automatically skip struct fields which are not `pub`
- Support for skipping variants or fields for `__str__` or `__repr__` differently with the `#[skip_str]` and `#[skip_repr]` attributes

> Note: When using the `StrRepr` macro. if `T` did not use `StrRepr`, it requires `T: Debug` for each `T` inside the item. The `Debug` trait is used for the outputs.

## Example
```rust
#[pyclass]
#[derive(Dir, StrRepr)]
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