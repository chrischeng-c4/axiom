# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_bytes_float_slice_dict_numeric_silent"
# subject = "cpython321.lang_bytes_float_slice_dict_numeric_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_bytes_float_slice_dict_numeric_silent.py"
# status = "filled"
# ///
"""cpython321.lang_bytes_float_slice_dict_numeric_silent: execute CPython 3.12 seed lang_bytes_float_slice_dict_numeric_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass divergence-spec fixture for the
# silent value-contract divergence of the
# `bytes` / `float.hex` / `float.fromhex` / `slice-of-collection`
# / `dict.fromkeys-as-instance-method` / int-numeric-accessor
# six-pack pinned to atomic 232:
# `bytes` (the documented `b.upper() / b.lower() / b.title() /
# b.capitalize() / b.swapcase() / b.isalpha() / b.isdigit() /
# b.isalnum() / b.isspace() / b.isupper() / b.islower() /
# b.zfill() / b.ljust() / b.rjust() / b.center() / b.partition() /
# b.rpartition() / b.splitlines() / b.rsplit() / b.expandtabs() /
# b.removeprefix() / b.removesuffix() / b.translate() / b.index() /
# b.rindex()` instance value contracts — mamba raises
# AttributeError at every one of those call sites because mamba's
# bytes type is the minimal split/replace/hex/fromhex/count/find/
# strip/decode/startswith/endswith subset), `float.hex` (the
# documented `(0.5).hex() == '0x1.0000000000000p-1'` IEEE 754
# normalized-mantissa value contract — mamba returns
# `'0x3fe0000000000000'`, the raw IEEE 754 bit pattern as a
# 64-bit hex integer, completely unrelated to the documented
# format), `float.fromhex` (the documented `float.fromhex(...)
# == 3.0` value contract — mamba returns the bytes singleton
# `b'\x01'` instead of a float), `[1,2,3,4,5][slice(1,5,2)]`
# (the documented "indexing a list with a slice() object
# returns a sub-list" value contract — mamba silently returns
# None), `'hello'[slice(1,4)]` (the same contract for str —
# mamba silently returns None), `{}.fromkeys(['a'])` (the
# documented "fromkeys is callable on a dict instance" surface
# — mamba binds fromkeys only as the dict classmethod and
# raises AttributeError on the instance), and the documented
# `(7).numerator == 7` / `(7).denominator == 1` int-numeric
# accessor contract — mamba silently returns None.
#
# Behavioral edges that CONFORM on mamba (str full method
# surface, list/dict/set/frozenset method surface, int
# bit_length/bit_count/to_bytes/from_bytes/abs/divmod/round/bin/hex/oct,
# float as_integer_ratio/is_integer/repr, range value+slice ops,
# tuple count/index/len, basic list+str slicing, bytes
# split/replace/hex/fromhex/count/find/strip/decode/startswith/
# endswith/len/rfind/join) are covered in the matching pass
# fixture
# `test_builtin_type_methods_value_ops`.
from typing import Any

bytes_mod: Any = bytes
float_mod: Any = float
dict_mod: Any = dict


_ledger: list[int] = []

# 1) bytes.upper / lower / title — instance method AttributeError
#    (mamba: bytes minimal surface lacks case methods)
try:
    _r = b"hello".upper()
    _ok = _r == b"HELLO"
except AttributeError:
    _ok = False
assert _ok == True; _ledger.append(1)
try:
    _r = b"HELLO".lower()
    _ok = _r == b"hello"
except AttributeError:
    _ok = False
assert _ok == True; _ledger.append(1)
try:
    _r = b"hello world".title()
    _ok = _r == b"Hello World"
except AttributeError:
    _ok = False
assert _ok == True; _ledger.append(1)
try:
    _r = b"hello".capitalize()
    _ok = _r == b"Hello"
except AttributeError:
    _ok = False
assert _ok == True; _ledger.append(1)
try:
    _r = b"Hello".swapcase()
    _ok = _r == b"hELLO"
except AttributeError:
    _ok = False
assert _ok == True; _ledger.append(1)

# 2) bytes.is* predicates — instance method AttributeError
#    (mamba: bytes minimal surface lacks classification methods)
try:
    _ok = b"abc".isalpha() == True
except AttributeError:
    _ok = False
assert _ok == True; _ledger.append(1)
try:
    _ok = b"123".isdigit() == True
except AttributeError:
    _ok = False
assert _ok == True; _ledger.append(1)
try:
    _ok = b"abc".isalnum() == True
except AttributeError:
    _ok = False
assert _ok == True; _ledger.append(1)
try:
    _ok = b"   ".isspace() == True
except AttributeError:
    _ok = False
assert _ok == True; _ledger.append(1)
try:
    _ok = b"ABC".isupper() == True
except AttributeError:
    _ok = False
assert _ok == True; _ledger.append(1)
try:
    _ok = b"abc".islower() == True
except AttributeError:
    _ok = False
assert _ok == True; _ledger.append(1)

# 3) bytes.zfill / ljust / rjust / center — pad methods
#    AttributeError (mamba: bytes minimal surface lacks pad methods)
try:
    _r = b"42".zfill(5)
    _ok = _r == b"00042"
except AttributeError:
    _ok = False
assert _ok == True; _ledger.append(1)
try:
    _r = b"abc".ljust(7, b"-")
    _ok = _r == b"abc----"
except AttributeError:
    _ok = False
assert _ok == True; _ledger.append(1)
try:
    _r = b"abc".rjust(7, b"-")
    _ok = _r == b"----abc"
except AttributeError:
    _ok = False
assert _ok == True; _ledger.append(1)
try:
    _r = b"abc".center(9, b"-")
    _ok = _r == b"---abc---"
except AttributeError:
    _ok = False
assert _ok == True; _ledger.append(1)

# 4) bytes.partition / rpartition / splitlines / rsplit —
#    split-family methods AttributeError
try:
    _r = b"hello world".partition(b"w")
    _ok = _r == (b"hello ", b"w", b"orld")
except AttributeError:
    _ok = False
assert _ok == True; _ledger.append(1)
try:
    _r = b"hello world".rpartition(b"o")
    _ok = _r == (b"hello w", b"o", b"rld")
except AttributeError:
    _ok = False
assert _ok == True; _ledger.append(1)
try:
    _r = b"a\nb\nc".splitlines()
    _ok = _r == [b"a", b"b", b"c"]
except AttributeError:
    _ok = False
assert _ok == True; _ledger.append(1)
try:
    _r = b"hello world".rsplit(b"o", 1)
    _ok = _r == [b"hello w", b"rld"]
except AttributeError:
    _ok = False
assert _ok == True; _ledger.append(1)

# 5) bytes.expandtabs / removeprefix / removesuffix / translate —
#    transform methods AttributeError
try:
    _r = b"a\tb".expandtabs(4)
    _ok = _r == b"a   b"
except AttributeError:
    _ok = False
assert _ok == True; _ledger.append(1)
try:
    _r = b"hello world".removeprefix(b"hello ")
    _ok = _r == b"world"
except AttributeError:
    _ok = False
assert _ok == True; _ledger.append(1)
try:
    _r = b"hello world".removesuffix(b" world")
    _ok = _r == b"hello"
except AttributeError:
    _ok = False
assert _ok == True; _ledger.append(1)
try:
    _r = b"abc".translate(bytes_mod.maketrans(b"a", b"A"))
    _ok = _r == b"Abc"
except AttributeError:
    _ok = False
assert _ok == True; _ledger.append(1)

# 6) bytes.index / rindex — search methods AttributeError
try:
    _r = b"hello world".index(b"world")
    _ok = _r == 6
except AttributeError:
    _ok = False
assert _ok == True; _ledger.append(1)
try:
    _r = b"hello world".rindex(b"world")
    _ok = _r == 6
except AttributeError:
    _ok = False
assert _ok == True; _ledger.append(1)

# 7) float.hex — IEEE 754 normalized-mantissa value contract
#    (mamba: returns raw IEEE 754 bit pattern '0x3fe0000000000000'
#    instead of '0x1.0000000000000p-1')
assert (0.5).hex() == "0x1.0000000000000p-1"; _ledger.append(1)
assert (2.0).hex() == "0x1.0000000000000p+1"; _ledger.append(1)
assert (1.0).hex() == "0x1.0000000000000p+0"; _ledger.append(1)

# 8) float.fromhex — value contract
#    (mamba: returns the bytes singleton b'\x01' instead of 3.0)
assert float_mod.fromhex("0x1.8p+1") == 3.0; _ledger.append(1)
assert float_mod.fromhex("0x1.0p+0") == 1.0; _ledger.append(1)

# 9) slice-of-collection — indexing-with-slice-object value contract
#    (mamba: silently returns None)
assert [1, 2, 3, 4, 5][slice(1, 5, 2)] == [2, 4]; _ledger.append(1)
assert "hello"[slice(1, 4)] == "ell"; _ledger.append(1)

# 10) dict.fromkeys as instance method — surface contract
#     (mamba: only the dict.fromkeys classmethod works; instance
#     binding raises AttributeError)
try:
    _r = {}.fromkeys(["a"])
    _ok = _r == {"a": None}
except AttributeError:
    _ok = False
assert _ok == True; _ledger.append(1)

# 11) int-numeric accessors — numerator / denominator value contract
#     (mamba: silently returns None)
assert (7).numerator == 7; _ledger.append(1)
assert (7).denominator == 1; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_bytes_float_slice_dict_numeric_silent {sum(_ledger)} asserts")
