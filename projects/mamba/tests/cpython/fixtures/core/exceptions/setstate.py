# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""Exception.__setstate__ and arbitrary attribute assignment (CPython 3.12 oracle)."""

# Exceptions accept arbitrary attribute assignment via __dict__.
e = Exception(42)
e.blah = 53
assert e.args == (42,)
assert e.blah == 53

# Reading an unset attribute raises AttributeError.
try:
    e.a
    raise AssertionError("expected AttributeError")
except AttributeError:
    pass
print("attr_assignment: blah=", e.blah)


# __setstate__ merges a dict of attributes (without touching args here).
e.__setstate__({"a": 1, "b": 2})
assert e.args == (42,)
assert e.blah == 53
assert e.a == 1
assert e.b == 2
print("setstate_merge: a=", e.a, "b=", e.b)


# A subsequent __setstate__ can also overwrite args and existing attrs.
e.__setstate__({"a": 11, "args": (1, 2, 3), "blah": 35})
assert e.args == (1, 2, 3)
assert e.blah == 35
assert e.a == 11
assert e.b == 2
print("setstate_overwrite: args=", e.args, "a=", e.a)


# Passing a non-dict state raises TypeError.
try:
    Exception(42).__setstate__(42)
    raise AssertionError("expected TypeError")
except TypeError as err:
    assert "dictionary" in str(err)
    print("invalid_setstate: TypeError ->", str(err))

print("setstate OK")
