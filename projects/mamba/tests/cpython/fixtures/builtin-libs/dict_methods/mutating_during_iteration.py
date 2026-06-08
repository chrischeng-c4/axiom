# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""dict_methods: changing dict size during iteration raises RuntimeError."""


# Inserting a new key while iterating the dict raises RuntimeError.
d = {1: 1}
try:
    for k in d:
        d[k + 1] = 1
    raise AssertionError("expected RuntimeError")
except RuntimeError:
    pass


# Delete-then-reinsert during iteration also raises (size changed mid-loop).
d = {0: 0}
try:
    for _ in d:
        del d[0]
        d[0] = 0
    raise AssertionError("expected RuntimeError")
except RuntimeError:
    pass


# Iterating a view while resizing the underlying dict raises too.
d = {1: 1}
try:
    for k in d.keys():
        d[k + 1] = 1
    raise AssertionError("expected RuntimeError (keys view)")
except RuntimeError:
    pass

d = {1: 1}
try:
    for v in d.values():
        d[len(d) + 1] = v
    raise AssertionError("expected RuntimeError (values view)")
except RuntimeError:
    pass

d = {1: 1}
try:
    for k, v in d.items():
        d[k + 1] = v
    raise AssertionError("expected RuntimeError (items view)")
except RuntimeError:
    pass


# Mutating a *value* in place (no size change) is allowed during iteration.
d = {1: 1, 2: 2, 3: 3}
for k in d:
    d[k] = d[k] * 10
assert d == {1: 10, 2: 20, 3: 30}

print("mutating_during_iteration OK")
