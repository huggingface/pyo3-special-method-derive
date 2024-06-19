"""
Install maturin in a virtual env: `pip install maturin[patchelf]`
Run: `maturin develop -r`

Set breakpoints for debugging, or simply run this program!
"""

import pyo3_smd_example

person = pyo3_smd_example.Person()
# Person(name="Name here", age=0, address=Address.House(country="Country here", city="City here", street="Street here", street_number=4294967295))
print(person)
# person.__dict__={'address': Address.House(country="Country here", city="City here", street="Street here", street_number=4294967295), 'name': 'Name here', 'age': 0}
print(f"{person.__dict__=}")
# person.address.__dict__={'city': 'City here', 'country': 'Country here', 'street': 'Street here', 'street_number': 4294967295}
print(f"{person.address.__dict__=}")
# person.name='Name here'
print(f"{person.name=}")
# person.address.country='Country here'
print(f"{person.address.country=}")
# dir(person)=['address', 'age', 'name']
print(f"{dir(person)=}")
# dir(person.address)=['city', 'country', 'street', 'street_number']
print(f"{dir(person.address)=}")