# RUN: parse

# Dict unpacking (PEP 448)
d1 = {"a": 1}
d2 = {"b": 2}
merged = {**d1, **d2}
merged = {**d1, "c": 3, **d2}
d3 = {"c": 3}
merged = {**d1, **d2, **d3}
