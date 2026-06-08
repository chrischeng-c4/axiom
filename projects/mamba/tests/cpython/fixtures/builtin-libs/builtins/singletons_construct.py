# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""builtins: singleton constructors and None.__ne__ identity contract."""

# Distilled from CPython Lib/test/test_builtin.py test_construct_singletons
# and test___ne__ (re-curated into one focused fixture).

# Calling the type of a singleton with no args returns that exact singleton.
for const in (None, Ellipsis, NotImplemented):
    tp = type(const)
    assert tp() is const

    # Passing any argument raises TypeError.
    try:
        tp(1, 2)
        raise AssertionError("expected TypeError")
    except TypeError:
        pass
    try:
        tp(a=1, b=2)
        raise AssertionError("expected TypeError")
    except TypeError:
        pass

# None.__ne__ compares by identity and defers cross-type comparisons.
assert None.__ne__(None) is False
assert None.__ne__(0) is NotImplemented
assert None.__ne__("abc") is NotImplemented

# The default object identity holds for the singletons.
assert None is None
assert (None == None) is True
assert (None != None) is False

print("singletons_construct OK")
