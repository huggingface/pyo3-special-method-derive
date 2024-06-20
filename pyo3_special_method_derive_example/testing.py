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

assert (
    str(person)
    == 'Person(name="Name here", age=0, address=Address.House(country="Country here", city="City here", street="Street here", street_number=4294967295))'
)

assert (
    str(person.__dict__)
    == "{'address': Address.House(country=\"Country here\", city=\"City here\", street=\"Street here\", street_number=4294967295), 'name': 'Name here', 'age': 0}"
)

assert person.address.__dict__ == {
    "city": "City here",
    "country": "Country here",
    "street": "Street here",
    "street_number": 4294967295,
}

assert person.name == "Name here"

assert person.address.country == "Country here"

assert dir(person) == ["address", "age", "name"]

assert dir(person.address) == ["city", "country", "street", "street_number"]
