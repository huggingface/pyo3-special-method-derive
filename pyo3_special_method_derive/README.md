# pyo3-special-method-derive-lib

This crate provides traits necessary for pyo3-special-method-derive.

This crate exports 2 traits which should be implemented for every type for which its field or variant is not skipped.

It also exports a macro to use the Debug and Display traits to generate a PyDebug and PyDisplay implementation.