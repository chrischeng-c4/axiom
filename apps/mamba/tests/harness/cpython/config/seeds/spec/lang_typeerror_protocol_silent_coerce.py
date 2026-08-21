# Spec seed for CPython TypeError / AttributeError / ValueError
# contract on the protocol-violation builtins that mamba silently
# coerces to a zero/None/empty value instead of dispatching the
# protocol fallback. Surface: CPython rejects (1) `iter(int)`,
# `len(int)`, `reversed(int)`, `sorted(int)` because int does not
# implement `__iter__` / `__len__` / `__reversed__`, (2) `hash([])`,
# `hash({})`, `hash(set())` because mutable containers are
# `__hash__ = None`, (3) `setattr(int, ...)` / `delattr(int, ...)` /
# `getattr(obj, "missing")` because the immutable int has no `__dict__`
# and `object()` has no arbitrary attributes, (4) `hasattr(int,
# non_str)` because the attribute-name argument must be a string, and
# (5) `format(int, "Z")` / `format(str, "d")` because the format spec
# is incompatible with the value type. Mamba 0.3.60 silently returns
# `None` / `0` / `[]` / `False` / hashed-via-id-or-content / formatted-
# as-default instead.
#
# Probes (every form CPython rejects, mamba silently coerces):
#   • iter(5)                        → mamba: None    (TypeError)
#   • len(5)                         → mamba: 0       (TypeError)
#   • reversed(5)                    → mamba: None    (TypeError)
#   • sorted(5)                      → mamba: []      (TypeError)
#   • hash([1, 2, 3])                → mamba: int     (TypeError)
#   • hash({'a': 1})                 → mamba: int     (TypeError)
#   • hash({1, 2, 3})                → mamba: int     (TypeError)
#   • setattr(5, "x", 1)             → mamba: None    (AttributeError)
#   • delattr(5, "x")                → mamba: None    (AttributeError)
#   • getattr(object(), "missing")   → mamba: None    (AttributeError)
#   • hasattr(5, 123)                → mamba: False   (TypeError)
#   • format(5, "Z")                 → mamba: '5'     (ValueError)
#   • format("hi", "d")              → mamba: '0'     (ValueError)
#
# CPython contract:
#   iter(int)              → TypeError("'int' object is not iterable");
#   len(int)               → TypeError("object of type 'int' has no
#                                  len()");
#   reversed(int)          → TypeError("'int' object is not
#                                  reversible");
#   sorted(int)            → TypeError("'int' object is not iterable");
#   hash(list/dict/set)    → TypeError("unhashable type: '<typename>'");
#   setattr(int, ...)      → AttributeError("'int' object has no
#                                  attribute 'x'");
#   delattr(int, ...)      → AttributeError("'int' object has no
#                                  attribute 'x'");
#   getattr(obj, miss)     → AttributeError("'object' object has no
#                                  attribute 'missing'");
#   hasattr(_, non_str)    → TypeError("attribute name must be string,
#                                  not 'int'");
#   format(int, "Z")       → ValueError("Unknown format code 'Z' for
#                                  object of type 'int'");
#   format(str, "d")       → ValueError("Unknown format code 'd' for
#                                  object of type 'str').
#
# `Any`-typed holders push the operand past static type-checkers
# (Pyright) and past mamba's compile-time argtype enforcement so the
# runtime divergence is what's exercised.
from typing import Any
_ledger: list[int] = []

_i: Any = 5
_obj: Any = object()
_int_name: Any = 123
_lst: Any = [1, 2, 3]
_dct: Any = {"a": 1}
_set: Any = {1, 2, 3}
_str: Any = "hi"
_bad_spec: Any = "Z"
_d_spec: Any = "d"

# iter(int) — int has no __iter__
try:
    _ = iter(_i)
    raise AssertionError("iter(int) must raise TypeError")
except TypeError:
    _ledger.append(1)

# len(int) — int has no __len__
try:
    _ = len(_i)
    raise AssertionError("len(int) must raise TypeError")
except TypeError:
    _ledger.append(1)

# reversed(int) — int has no __reversed__
try:
    _ = reversed(_i)
    raise AssertionError("reversed(int) must raise TypeError")
except TypeError:
    _ledger.append(1)

# sorted(int) — int has no __iter__
try:
    _ = sorted(_i)
    raise AssertionError("sorted(int) must raise TypeError")
except TypeError:
    _ledger.append(1)

# hash(list) — list is unhashable
try:
    _ = hash(_lst)
    raise AssertionError("hash(list) must raise TypeError")
except TypeError:
    _ledger.append(1)

# hash(dict) — dict is unhashable
try:
    _ = hash(_dct)
    raise AssertionError("hash(dict) must raise TypeError")
except TypeError:
    _ledger.append(1)

# hash(set) — set is unhashable
try:
    _ = hash(_set)
    raise AssertionError("hash(set) must raise TypeError")
except TypeError:
    _ledger.append(1)

# setattr(int, "x", 1) — int has no __dict__
try:
    setattr(_i, "x", 1)
    raise AssertionError("setattr(int, ...) must raise AttributeError")
except AttributeError:
    _ledger.append(1)

# delattr(int, "x") — int has no __dict__
try:
    delattr(_i, "x")
    raise AssertionError("delattr(int, ...) must raise AttributeError")
except AttributeError:
    _ledger.append(1)

# getattr(object(), "missing") — no default → AttributeError
try:
    _ = getattr(_obj, "no_such_attr_xyz_qqq")
    raise AssertionError("getattr(obj, missing) must raise AttributeError")
except AttributeError:
    _ledger.append(1)

# hasattr(_, int_name) — attribute name must be str
try:
    _ = hasattr(_i, _int_name)
    raise AssertionError("hasattr(_, int_name) must raise TypeError")
except TypeError:
    _ledger.append(1)

# format(int, "Z") — bad format code for int
try:
    _ = format(_i, _bad_spec)
    raise AssertionError("format(int, 'Z') must raise ValueError")
except ValueError:
    _ledger.append(1)

# format("hi", "d") — bad format code for str
try:
    _ = format(_str, _d_spec)
    raise AssertionError("format(str, 'd') must raise ValueError")
except ValueError:
    _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_typeerror_protocol_silent_coerce {sum(_ledger)} asserts")
