# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""exec(src, ns) namespace binding (CPython 3.12 oracle)."""

ns: dict = {}
exec("x = 42", ns)
print("x:", ns.get("x"))

print("written:", "x" in ns)

ns2: dict = {}
exec("y = 'hi'; z = [1, 2, 3]", ns2)
print("y:", ns2.get("y"))
print("z:", ns2.get("z"))
