# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
d = {"a": 1}
print(d.get("a"))
print(d.get("b"))
print(d.get("b", 42))
d.setdefault("c", 3)
print(d["c"])
d.setdefault("c", 99)
print(d["c"])
