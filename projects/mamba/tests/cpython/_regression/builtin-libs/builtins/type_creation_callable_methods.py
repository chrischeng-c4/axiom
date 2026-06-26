# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""builtins.type: callable namespace entries bind and dispatch as methods."""

C = type("C", (), {"f": lambda self: 42})
assert C().f() == 42


def g(self):
    return 42


D = type("D", (), {"g": g})
assert D().g() == 42

E = type("E", (), {"__len__": lambda self: 5})
assert len(E()) == 5

print("type_creation_callable_methods OK")
