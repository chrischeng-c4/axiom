"""Behavior contract for builtins.bool.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

# Rule 1: bool() with no args returns False
assert bool() is False, "bool() expected False"

# Rule 2: bool(truthy int)
assert bool(1) is True, "bool(1) expected True"
assert bool(-1) is True, "bool(-1) expected True"

# Rule 3: bool(falsy int)
assert bool(0) is False, "bool(0) expected False"

# Rule 4: bool(str)
assert bool("x") is True, "bool('x') expected True"
assert bool("") is False, "bool('') expected False"

# Rule 5: bool(list)
assert bool([1]) is True, "bool([1]) expected True"
assert bool([]) is False, "bool([]) expected False"

# Rule 6: bool(None) is False
assert bool(None) is False, "bool(None) expected False"

# Rule 7: True and False are singletons (identity, not just equality)
assert True is True, "True is not True"
assert False is False, "False is not False"

# Rule 8: bool arithmetic (True acts as 1, False as 0)
assert True + True == 2, f"True + True = {True + True!r}"
assert True + False == 1, f"True + False = {True + False!r}"
assert False + False == 0, f"False + False = {False + False!r}"

# Rule 9: not operator
assert not True is False, "not True expected False"
assert not False is True, "not False expected True"

# Rule 10: bool comparison operators
assert (True > False) == True, "True > False expected True"
assert (False < True) == True, "False < True expected True"

# Rule 11: bool converts via __bool__
class _Truthy:
    def __bool__(self):
        return True
class _Falsy:
    def __bool__(self):
        return False
assert bool(_Truthy()) is True
assert bool(_Falsy()) is False

# Rule 12: bool(float)
assert bool(0.0) is False
assert bool(1.5) is True

print("behavior OK")
