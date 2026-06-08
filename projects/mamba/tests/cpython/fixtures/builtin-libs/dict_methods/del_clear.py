# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
d = {"a": 1, "b": 2, "c": 3}
del d["b"]
print(sorted(d.items()))
e = d.copy()
d.clear()
print(len(d))
print(sorted(e.items()))
