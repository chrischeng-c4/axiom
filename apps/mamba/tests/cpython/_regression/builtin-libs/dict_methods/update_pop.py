# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
d = {"a": 1}
d.update({"b": 2, "c": 3})
print(sorted(d.items()))
v = d.pop("b")
print(v)
print(sorted(d.items()))
print(d.pop("x", 99))
