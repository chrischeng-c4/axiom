# Operational AssertionPass seed for SILENT divergences across the
# core built-in language quintet pinned by atomic 156: `bytes`
# (the documented `upper` / `lower` / `title` / `swapcase` /
# `capitalize` / `isalpha` / `isdigit` case-conversion +
# classification helpers), `bytearray` (the documented `list()`
# byte-iteration + `bytes()` conversion protocol), instance
# `__class__.__name__` (the documented runtime-class identity
# attribute), the dunder-add surface (`str.__add__` /
# `tuple.__add__` / `bytes.__add__`), the comprehension
# multi-`if` syntax (`[x for x in seq if cond1 if cond2]`), and
# built-in exception class identity (`ValueError.__name__` /
# `KeyError.__name__` / `TypeError.__name__` / `Exception.__name__`
# / `BaseException.__name__`).
#
# The matching subset (bytes indexing + find / replace / split /
# strip / startswith / endswith / hex / count / fromhex,
# bytearray append / extend / reverse / item-assign via
# bytearray-equality, numeric int / float / hex / oct / bin /
# chr / ord / abs / round / divmod / pow conversions, complex
# arithmetic + .real / .imag / .conjugate, iter + next default
# + StopIteration, f-string format-spec mini-language, type /
# isinstance / issubclass identity, simple comprehensions +
# walrus + and-conjoined predicate, exception args + str(e))
# is covered by `test_builtin_numeric_bytes_iter_value_ops`;
# this fixture pins the CPython-only contracts that mamba
# currently elides.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • b"Hello".upper() == b"HELLO" — case-conversion helper
#     (mamba: AttributeError, 'bytes' object has no attribute
#     'upper');
#   • b"Hello".lower() == b"hello" (mamba: AttributeError);
#   • b"Hello".title() == b"Hello" (mamba: AttributeError);
#   • b"Hello".swapcase() == b"hELLO" (mamba: AttributeError);
#   • b"Hello".capitalize() == b"Hello" (mamba: AttributeError);
#   • b"Hello".isalpha() is True — alphabetic classification
#     (mamba: AttributeError);
#   • b"123".isdigit() is True (mamba: AttributeError);
#   • list(bytearray(b"abc")) == [97, 98, 99] — byte iteration
#     protocol (mamba: returns []);
#   • bytes(bytearray(b"abc")) == b"abc" — bytearray → bytes
#     conversion (mamba: returns b"");
#   • (42).__class__.__name__ == "int" — runtime-class identity
#     attribute on integer instances (mamba: returns None);
#   • (3.14).__class__.__name__ == "float" (mamba: None);
#   • "x".__class__.__name__ == "str" (mamba: None);
#   • [].__class__.__name__ == "list" (mamba: None);
#   • {}.__class__.__name__ == "dict" (mamba: None);
#   • ().__class__.__name__ == "tuple" (mamba: None);
#   • "a".__add__("b") == "ab" — dunder-add on string (mamba:
#     AttributeError, 'str' object has no attribute '__add__');
#   • (1,).__add__((2,)) == (1, 2) — dunder-add on tuple
#     (mamba: AttributeError);
#   • b"a".__add__(b"b") == b"ab" — dunder-add on bytes
#     (mamba: AttributeError);
#   • [x for x in range(10) if x % 2 == 0 if x > 4] == [6, 8] —
#     multi-`if` comprehension filtering (mamba: silently drops
#     the second `if` clause, returns [0, 2, 4, 6, 8]);
#   • ValueError.__name__ == "ValueError" — built-in exception
#     class identity (mamba: returns None);
#   • KeyError.__name__ == "KeyError" (mamba: None);
#   • TypeError.__name__ == "TypeError" (mamba: None);
#   • Exception.__name__ == "Exception" (mamba: None);
#   • BaseException.__name__ == "BaseException" (mamba: None).
from typing import Any

# Built-in callables retyped as `Any` to bypass Pyright stub-driven
# narrowing — every spec contract below probes documented public
# behaviour on built-in singletons / classes that mamba's bundled
# type stubs do not surface accurately.
_bytes_upper: Any = b"Hello".upper
_bytes_lower: Any = b"Hello".lower
_bytes_title: Any = b"Hello".title
_bytes_swapcase: Any = b"Hello".swapcase
_bytes_capitalize: Any = b"Hello".capitalize
_bytes_isalpha: Any = b"Hello".isalpha
_bytes_isdigit: Any = b"123".isdigit
_str_add: Any = "a".__add__
_tuple_add: Any = (1,).__add__
_bytes_add: Any = b"a".__add__


_ledger: list[int] = []

# 1) bytes — case-conversion helpers
assert _bytes_upper() == b"HELLO"; _ledger.append(1)
assert _bytes_lower() == b"hello"; _ledger.append(1)
assert _bytes_title() == b"Hello"; _ledger.append(1)
assert _bytes_swapcase() == b"hELLO"; _ledger.append(1)
assert _bytes_capitalize() == b"Hello"; _ledger.append(1)

# 2) bytes — character classification helpers
assert _bytes_isalpha() == True; _ledger.append(1)
assert _bytes_isdigit() == True; _ledger.append(1)

# 3) bytearray — list() iteration + bytes() conversion
assert list(bytearray(b"abc")) == [97, 98, 99]; _ledger.append(1)
assert bytes(bytearray(b"abc")) == b"abc"; _ledger.append(1)

# 4) Instance __class__.__name__ — built-in singletons
_int_cls: Any = (42).__class__
_flt_cls: Any = (3.14).__class__
_str_cls: Any = "x".__class__
_lst_cls: Any = [].__class__
_dct_cls: Any = {}.__class__
_tup_cls: Any = ().__class__
assert _int_cls.__name__ == "int"; _ledger.append(1)
assert _flt_cls.__name__ == "float"; _ledger.append(1)
assert _str_cls.__name__ == "str"; _ledger.append(1)
assert _lst_cls.__name__ == "list"; _ledger.append(1)
assert _dct_cls.__name__ == "dict"; _ledger.append(1)
assert _tup_cls.__name__ == "tuple"; _ledger.append(1)

# 5) Dunder __add__ on immutable sequence builtins
assert _str_add("b") == "ab"; _ledger.append(1)
assert _tuple_add((2,)) == (1, 2); _ledger.append(1)
assert _bytes_add(b"b") == b"ab"; _ledger.append(1)

# 6) Multi-`if` comprehension — both clauses must filter
assert [x for x in range(10) if x % 2 == 0 if x > 4] == [6, 8]; _ledger.append(1)

# 7) Built-in exception class __name__
_value_err: Any = ValueError
_key_err: Any = KeyError
_type_err: Any = TypeError
_exc_cls: Any = Exception
_base_exc: Any = BaseException
assert _value_err.__name__ == "ValueError"; _ledger.append(1)
assert _key_err.__name__ == "KeyError"; _ledger.append(1)
assert _type_err.__name__ == "TypeError"; _ledger.append(1)
assert _exc_cls.__name__ == "Exception"; _ledger.append(1)
assert _base_exc.__name__ == "BaseException"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_bytes_class_comp_silent {sum(_ledger)} asserts")
