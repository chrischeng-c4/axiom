# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""A class __dict__ is a read-only mappingproxy view (CPython 3.12 oracle)."""


class C:
    answer = 42

    def meth(self):
        return self.answer


proxy = C.__dict__

# The class namespace is exposed as a mappingproxy, not a plain dict.
assert type(proxy).__name__ == "mappingproxy"
assert repr(proxy).startswith("mappingproxy(")
assert repr(proxy).endswith(")")

# It supports read access, membership, and iteration.
assert proxy["answer"] == 42
assert "meth" in proxy
assert "answer" in proxy
keys = set(proxy.keys())
assert "answer" in keys and "meth" in keys

# keys()/values()/items() are views, not lists.
assert not isinstance(proxy.keys(), list)
assert not isinstance(proxy.values(), list)
assert dict(proxy.items())["answer"] == 42

# It is read-only: assignment and deletion raise TypeError.
try:
    proxy["new"] = 1
    print("set: no_raise")
except TypeError as e:
    print("set: TypeError", str(e)[:40])

try:
    del proxy["answer"]
    print("del: no_raise")
except TypeError:
    print("del: TypeError")

# Attributes are still settable through the class itself.
C.added = 7
assert C.__dict__["added"] == 7

print("mappingproxy OK")
