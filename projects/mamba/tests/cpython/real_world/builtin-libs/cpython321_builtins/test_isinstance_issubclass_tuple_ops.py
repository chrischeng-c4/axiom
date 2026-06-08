# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "cpython321_builtins"
# dimension = "real_world"
# case = "test_isinstance_issubclass_tuple_ops"
# subject = "cpython321.test_isinstance_issubclass_tuple_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_isinstance_issubclass_tuple_ops.py"
# status = "filled"
# ///
"""cpython321.test_isinstance_issubclass_tuple_ops: execute CPython 3.12 seed test_isinstance_issubclass_tuple_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for `isinstance` / `issubclass` with
# a tuple of types — surface not covered by `test_type_introspection_ops`
# (single-type isinstance/issubclass only). This seed asserts:
#   * isinstance(value, (T1, T2, ...)) returns True iff `value` is
#     an instance of any listed type;
#   * issubclass(cls, (T1, T2, ...)) returns True iff `cls` is a
#     subclass of any listed type;
#   * The numeric tower: bool is a subclass of int; int is not a
#     subclass of float; both int and float are subclasses of object;
#   * isinstance correctly distinguishes between sequence types
#     (list vs tuple);
#   * type(x) returns the canonical type for fundamental values.
_ledger: list[int] = []

# isinstance with tuple of types — at least one match → True
assert isinstance(5, (int, float)); _ledger.append(1)
assert isinstance(5.0, (int, float)); _ledger.append(1)
assert isinstance("x", (int, str)); _ledger.append(1)
assert isinstance([], (list, tuple)); _ledger.append(1)
assert isinstance((1,), (list, tuple)); _ledger.append(1)
assert isinstance({}, (dict, set)); _ledger.append(1)
assert isinstance(set(), (dict, set)); _ledger.append(1)

# isinstance with tuple of types — no match → False
assert not isinstance("x", (int, float)); _ledger.append(1)
assert not isinstance([], (int, str)); _ledger.append(1)
assert not isinstance(3.14, (str, list, tuple)); _ledger.append(1)
assert not isinstance(None, (int, float, str)); _ledger.append(1)

# Three-element tuple
assert isinstance(5, (str, int, float)); _ledger.append(1)
assert isinstance(3.14, (str, int, float)); _ledger.append(1)
assert isinstance("x", (str, int, float)); _ledger.append(1)
# Four-element tuple
assert isinstance([1, 2], (int, str, list, dict)); _ledger.append(1)

# isinstance on bool — bool is a subclass of int, so True isinstance int
assert isinstance(True, int); _ledger.append(1)
assert isinstance(False, int); _ledger.append(1)
assert isinstance(True, (int, str)); _ledger.append(1)
# But not isinstance of float
assert not isinstance(True, float); _ledger.append(1)

# issubclass with single type
assert issubclass(bool, int); _ledger.append(1)
assert issubclass(int, object); _ledger.append(1)
assert issubclass(float, object); _ledger.append(1)
assert issubclass(str, object); _ledger.append(1)
# Not all subclass relations hold
assert not issubclass(str, int); _ledger.append(1)
assert not issubclass(int, float); _ledger.append(1)
assert not issubclass(list, tuple); _ledger.append(1)

# issubclass with tuple of base classes
assert issubclass(int, (int, float)); _ledger.append(1)
assert issubclass(bool, (int, float)); _ledger.append(1)  # bool < int < (int,float)
assert issubclass(float, (int, float)); _ledger.append(1)
assert not issubclass(str, (int, float)); _ledger.append(1)
assert issubclass(list, (list, tuple, dict)); _ledger.append(1)

# type() returns the canonical type — compare by __name__ to avoid
# the identity quirk on type objects in mamba
assert type(5).__name__ == "int"; _ledger.append(1)
assert type("x").__name__ == "str"; _ledger.append(1)
assert type([]).__name__ == "list"; _ledger.append(1)
assert type((1,)).__name__ == "tuple"; _ledger.append(1)
assert type({}).__name__ == "dict"; _ledger.append(1)
assert type(True).__name__ == "bool"; _ledger.append(1)
assert type(3.14).__name__ == "float"; _ledger.append(1)

# type() of int and bool have different names even though bool < int
assert type(1).__name__ == "int"; _ledger.append(1)
assert type(True).__name__ == "bool"; _ledger.append(1)
assert type(True).__name__ != type(1).__name__; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_isinstance_issubclass_tuple_ops {sum(_ledger)} asserts")
