# pyo3-special-method-derive

This crate enables you to automatically derive Python dunder methods for your Rust crate.

The following methods may be derived:
- `__str__`
- `__repr__`
- `__dir__`

Coming soon:
- `__dict__`

## PyO3 feature note
To use `pyo3-special-method-derive`, you should enable the `multiple-pymethods` feature on PyO3:
```
pyo3 = { version = "0.21", features = ["multiple-pymethods"] }
```