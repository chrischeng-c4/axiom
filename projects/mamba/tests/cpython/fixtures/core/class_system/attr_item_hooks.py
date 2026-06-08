# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""__getattr__/__setattr__/__delattr__ and item protocol hooks (CPython 3.12)."""


class Hooked:
    def __init__(self):
        # Bypass our own __setattr__ to seed real storage.
        object.__setattr__(self, "_store", {})

    def __getattr__(self, name):
        # Only called when normal lookup fails.
        if name == "magic":
            return "from_getattr"
        raise AttributeError(name)

    def __setattr__(self, name, value):
        self._store[name] = value

    def __delattr__(self, name):
        del self._store[name]

    def __getitem__(self, key):
        return ("getitem", key)

    def __setitem__(self, key, value):
        self._store[("item", key)] = value

    def __delitem__(self, key):
        self._store[("del", key)] = True


h = Hooked()

# __getattr__ only fires for absent names.
assert h.magic == "from_getattr"

# __setattr__ intercepts every assignment.
h.foo = 12
assert h._store["foo"] == 12

# __delattr__ intercepts deletion.
del h.foo
assert "foo" not in h._store

# Item protocol, including slice objects passed through unchanged.
assert h[7] == ("getitem", 7)
assert h[1:5] == ("getitem", slice(1, 5))
h[3] = "x"
assert h._store[("item", 3)] == "x"
del h[3]
assert h._store[("del", 3)] is True

# __getattr__ that raises AttributeError surfaces normally.
try:
    h.nope
    print("missing: no_raise")
except AttributeError:
    print("missing: AttributeError")


# __getattribute__ intercepts every attribute access, not just misses.
class Watch:
    def __getattribute__(self, name):
        if name == "shadow":
            return 99
        return object.__getattribute__(self, name)


w = Watch()
w.real = 1
assert w.real == 1
assert w.shadow == 99

print("attr_item_hooks OK")
