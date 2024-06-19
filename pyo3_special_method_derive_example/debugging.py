"""
Install maturin in a virtual env: `pip install maturin[patchelf]`
Run: `matuin develop -r`

Set breakpoints for debugging, or simply run this program!
"""

import pyo3_smd_example

person = pyo3_smd_example.Person()

print(person)