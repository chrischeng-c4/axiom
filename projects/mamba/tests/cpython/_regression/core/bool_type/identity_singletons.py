# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""core/bool_type: predicates return the True/False singletons (not just
truthy/falsy values). `is True` / `is False` is stricter than `==`; it pins
down that these operations yield the canonical bool objects."""

import operator

# isinstance / issubclass yield real bools. Note 1 is an int, not a bool,
# so isinstance(1, bool) is False even though isinstance(True, int) is True.
assert isinstance(True, bool) is True and isinstance(False, bool) is True
assert isinstance(True, int) is True and isinstance(False, int) is True
assert isinstance(1, bool) is False and isinstance(0, bool) is False
assert issubclass(bool, int) is True
assert issubclass(int, bool) is False

# Membership, attribute and callability probes return singletons.
assert (1 in {1: 1}) is True and (1 in {}) is False
assert (2 in [1, 2, 3]) is True and (9 in [1, 2, 3]) is False
assert hasattr([], "append") is True and hasattr([], "wobble") is False
assert callable(len) is True and callable(1) is False

# A representative slice of str predicate methods returns singletons.
assert "xyz".startswith("x") is True and "xyz".startswith("z") is False
assert "xyz".endswith("z") is True and "xyz".endswith("x") is False
assert "0123".isdigit() is True and "xyz".isdigit() is False
assert "xyz".isalpha() is True and "@#$".isalpha() is False
assert "xyz".islower() is True and "XYZ".islower() is False

# The operator module mirrors the syntactic forms and returns singletons.
assert operator.truth(1) is True and operator.truth(0) is False
assert operator.not_(0) is True and operator.not_(1) is False
assert operator.contains([1], 1) is True and operator.contains([], 1) is False
assert operator.lt(0, 1) is True and operator.lt(0, 0) is False
assert operator.is_(True, True) is True and operator.is_(True, False) is False
assert operator.is_not(True, False) is True and operator.is_not(True, True) is False

print("identity_singletons OK")
