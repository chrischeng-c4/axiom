# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""core/bool_type: language-area behavior asserts (CPython 3.12 oracle)."""

# Generic language behavior checks.
assert 1 + 1 == 2
assert "a" + "b" == "ab"
assert isinstance(True, int)
assert isinstance(1, int)
assert type(()) is tuple
assert type([]) is list
assert type({}) is dict
assert len("abc") == 3
assert list(range(3)) == [0, 1, 2]

# bool is a subclass of int: int/float/complex conversions agree numerically,
# but produce real int/float/complex objects (not the bool singletons).
assert int(True) == 1 and int(False) == 0
assert int(True) is not True and int(False) is not False
assert float(True) == 1.0 and float(False) == 0.0
assert float(True) is not True
assert complex(True) == 1 + 0j and complex(False) == 0j
assert complex(True) == True and complex(False) == False

# Inherited int attributes: .real is the value, .imag is zero, both plain ints.
assert True.real == 1 and True.imag == 0
assert False.real == 0 and False.imag == 0
assert type(True.real) is int and type(True.imag) is int

# Bitwise &, |, ^ on two bools stay bool; mixing with int widens to int.
assert (True & True) is True and (True | True) is True
assert (True ^ True) is False and (False ^ True) is True
assert (True & 1) == 1 and not isinstance(True & 1, bool)
assert (True ^ 1) == 0 and not isinstance(True ^ 1, bool)

# Text forms: str/repr are the capitalised names; repr round-trips via eval.
assert str(True) == "True" and str(False) == "False"
assert repr(True) == "True" and repr(False) == "False"
assert eval(repr(True)) is True and eval(repr(False)) is False
assert "%d" % True == "1" and "%d" % False == "0"
assert "%x" % True == "1" and "%x" % False == "0"
assert format(True) == "True" and f"{False}" == "False"

# bool.from_bytes is the inherited int classmethod; result is a real bool.
assert bool.from_bytes(b"\x00" * 8, "big") is False
assert bool.from_bytes(b"abcd", "little") is True

# Construction edges: zero-arg bool() is False; every type object is truthy.
assert bool() is False
assert all(bool(t) is True
           for t in [bool, int, float, complex, str, list, dict, set,
                     tuple, object, type])

# bool.__new__ mirrors bool(): defaults to False, coerces its argument.
assert bool.__new__(bool) is False
assert bool.__new__(bool, 1) is True and bool.__new__(bool, 0) is False
assert bool.__new__(bool, True) is True and bool.__new__(bool, False) is False

print("behavior OK")
