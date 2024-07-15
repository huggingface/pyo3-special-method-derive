"""
Install maturin in a virtual env: `pip install maturin[patchelf]`
Run: `maturin develop -r`

Set breakpoints for debugging, or simply run this program!
"""

import pyo3_smd_example

person = pyo3_smd_example.Person(
    name="Name here",
    age=0,
    country="Country here",
    city="City here",
    street="Street here",
    street_number=4294967295,
)

print(str(person))

print(person.__dict__)

assert person.address.__dict__ == {
    "city": "City here",
    "country": "Country here",
    "street": "Street here",
    "street_number": 4294967295,
}

print(person.name)

print(person.address.country)

print(dir(person))

print(dir(person.address))