import pyo3_smd_example

# Create a new city
london = pyo3_smd_example.PyCity("London")

# Create a person living in London
person = pyo3_smd_example.Person(
    name="John Doe",
    age=30,
    country="UK",
    city=london,
    street="Baker Street",
    street_number=221
)

# Check if the address is occupied in London
address_key = "Baker Street-221"
is_occupied = london.is_address_occupied(address_key)
print(f"Address {address_key} occupied: {is_occupied}")  # Should print: True

# Change the person's address
new_york = pyo3_smd_example.PyCity("New York")
person.change_address(
    new_country="USA",
    new_city=new_york,
    new_street="5th Avenue",
    new_street_number=100
)

# Check if the old address is freed in London
is_occupied_old = london.is_address_occupied(address_key)
print(f"Old address {address_key} occupied: {is_occupied_old}")  # Should print: False

# Check if the new address is occupied in New York
new_address_key = "5th Avenue-100"
is_occupied_new = new_york.is_address_occupied(new_address_key)
print(f"New address {new_address_key} occupied: {is_occupied_new}")  # Should print: True

# Print the person's new address
new_address = person.get_address()
print(f"John's new address: {new_address}")  # Should print the new full address