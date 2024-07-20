# Example of [pyo3_special_method_derive](https://crates.io/crates/pyo3_special_method_derive)

This is an example of the pyo3_special_method_derive crate, demonstrating the `__dir__`, `__dict__`, `__str__`/`__repr__`, and `__getattr__` features on structs and enums in both printing and debugging use cases.

## Install
- Install maturin in a virtual env: `pip install maturin[patchelf]`
- Install the example: `matuin develop -r`

## Example
```py
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
```