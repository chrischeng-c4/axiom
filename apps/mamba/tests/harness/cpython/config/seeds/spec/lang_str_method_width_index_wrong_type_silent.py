# Spec seed for CPython TypeError contract on the string-method
# / list-mutator / bytes-split corners where the int / index / count
# argument must be a real `int` (or sep argument must be a real `str`
# / `bytes`) but mamba silently coerces.
#
# Surface: CPython rejects (1) `str.zfill(non_int)` /
# `str.ljust(non_int)` / `str.rjust(non_int)` /
# `str.center(non_int_width)` because these widths route through
# `__index__` and floats / None don't implement it — TypeError
# ("'<type>' object cannot be interpreted as an integer"); (2)
# `str.partition(non_str)` / `str.rpartition(non_str)` because the
# separator must be exactly `str` and non-str routes raise — TypeError
# ("must be str, not <type>"); (3) `str.replace(old, new, non_int)`
# because the count parameter routes through `__index__` — TypeError;
# (4) `list.insert(non_int_idx, obj)` / `list.pop(non_int_idx)`
# because index parameters route through `__index__` — TypeError; (5)
# `bytes.split(non_bytes_sep)` because the separator must be bytes or
# None — TypeError.
#
# Mamba accepts every form and silently no-ops / returns the original
# value / returns None, so code like `field.zfill(width_from_config)`
# where `width_from_config` is accidentally `float` (e.g., parsed
# from JSON without int-cast) silently leaves the field
# zero-pad-free, and `cache.insert(key_hash, entry)` where `key_hash`
# is `float` silently no-ops the insertion.
#
# Existing lang_typeerror_str_bytes_method_silent.py covers
# `startswith / endswith / count / split / replace.find`-style cases.
# Existing lang_typeerror_method_argtype.py covers `str.center fillchar
# int`. Existing lang_typeerror_dict_type_constructor_silent.py
# covers `dict.fromkeys(non_iter)`. This seed covers the FRESH
# divergence family: width/index/count arguments routed through
# `__index__` (zfill/ljust/rjust/center-WIDTH/replace-COUNT/list.insert
# /list.pop) and the partition/rpartition separator-type checks.
#
# Probes (every form CPython rejects, mamba silently coerces):
#   • 'abc'.zfill(3.14)              → mamba: 'abc'      (TypeError)
#   • 'abc'.zfill(None)              → mamba: 'abc'      (TypeError)
#   • 'abc'.ljust(3.14)              → mamba: 'abc'      (TypeError)
#   • 'abc'.ljust(None)              → mamba: 'abc'      (TypeError)
#   • 'abc'.rjust(3.14)              → mamba: 'abc'      (TypeError)
#   • 'abc'.rjust(None)              → mamba: 'abc'      (TypeError)
#   • 'abc'.center(3.14)             → mamba: 'abc'      (TypeError)
#   • 'abc'.center(None)             → mamba: 'abc'      (TypeError)
#   • 'aaaa'.replace('a','b',3.14)   → mamba: 'bbbb'     (TypeError)
#   • 'aaaa'.replace('a','b',None)   → mamba: 'bbbb'     (TypeError)
#   • [1,2,3].insert(3.14, 99)       → mamba: no-op      (TypeError)
#   • [1,2,3].insert(None, 99)       → mamba: no-op      (TypeError)
#   • [1,2,3].pop(3.14)              → mamba: no-op      (TypeError)
#   • [1,2,3].pop('x')               → mamba: no-op      (TypeError)
#   • 'abc'.partition(5)             → mamba: silent     (TypeError)
#   • 'abc'.partition(None)          → mamba: silent     (TypeError)
#   • 'abc'.rpartition(5)            → mamba: silent     (TypeError)
#   • b'abc'.split(5)                → mamba: silent     (TypeError)
#
# CPython contract (uniform across every form):
#   str.zfill / .ljust / .rjust / .center(non_int_width)
#       → TypeError("'<type>' object cannot be interpreted as an
#                    integer");
#   str.replace(old, new, non_int_count)
#       → TypeError("'<type>' object cannot be interpreted as an
#                    integer");
#   list.insert(non_int_idx, obj) / list.pop(non_int_idx)
#       → TypeError("'<type>' object cannot be interpreted as an
#                    integer");
#   str.partition / .rpartition(non_str_sep)
#       → TypeError("must be str, not <type>");
#   bytes.split(non_bytes_sep)
#       → TypeError("a bytes-like object is required, not '<type>'").
#
# `Any`-typed holders push the operand past static type-checkers
# (Pyright) and past mamba's compile-time argtype enforcement so
# the runtime divergence is what's exercised.
from typing import Any
_ledger: list[int] = []

_int: Any = 5
_none: Any = None
_flt: Any = 3.14
_str: Any = "x"

# 'abc'.zfill(3.14) — width via __index__
try:
    _ = "abc".zfill(_flt)
    raise AssertionError("'abc'.zfill(3.14) must raise TypeError")
except TypeError:
    _ledger.append(1)

# 'abc'.zfill(None)
try:
    _ = "abc".zfill(_none)
    raise AssertionError("'abc'.zfill(None) must raise TypeError")
except TypeError:
    _ledger.append(1)

# 'abc'.ljust(3.14)
try:
    _ = "abc".ljust(_flt)
    raise AssertionError("'abc'.ljust(3.14) must raise TypeError")
except TypeError:
    _ledger.append(1)

# 'abc'.ljust(None)
try:
    _ = "abc".ljust(_none)
    raise AssertionError("'abc'.ljust(None) must raise TypeError")
except TypeError:
    _ledger.append(1)

# 'abc'.rjust(3.14)
try:
    _ = "abc".rjust(_flt)
    raise AssertionError("'abc'.rjust(3.14) must raise TypeError")
except TypeError:
    _ledger.append(1)

# 'abc'.rjust(None)
try:
    _ = "abc".rjust(_none)
    raise AssertionError("'abc'.rjust(None) must raise TypeError")
except TypeError:
    _ledger.append(1)

# 'abc'.center(3.14) — width param (existing covers fillchar)
try:
    _ = "abc".center(_flt)
    raise AssertionError("'abc'.center(3.14) must raise TypeError")
except TypeError:
    _ledger.append(1)

# 'abc'.center(None)
try:
    _ = "abc".center(_none)
    raise AssertionError("'abc'.center(None) must raise TypeError")
except TypeError:
    _ledger.append(1)

# 'aaaa'.replace('a','b', 3.14) — count via __index__
try:
    _ = "aaaa".replace("a", "b", _flt)
    raise AssertionError("str.replace count=float must raise TypeError")
except TypeError:
    _ledger.append(1)

# 'aaaa'.replace('a','b', None)
try:
    _ = "aaaa".replace("a", "b", _none)
    raise AssertionError("str.replace count=None must raise TypeError")
except TypeError:
    _ledger.append(1)

# [1,2,3].insert(3.14, 99) — index via __index__
try:
    _l: Any = [1, 2, 3]
    _l.insert(_flt, 99)
    raise AssertionError("list.insert(3.14, 99) must raise TypeError")
except TypeError:
    _ledger.append(1)

# [1,2,3].insert(None, 99)
try:
    _l: Any = [1, 2, 3]
    _l.insert(_none, 99)
    raise AssertionError("list.insert(None, 99) must raise TypeError")
except TypeError:
    _ledger.append(1)

# [1,2,3].pop(3.14) — index via __index__
try:
    _l: Any = [1, 2, 3]
    _ = _l.pop(_flt)
    raise AssertionError("list.pop(3.14) must raise TypeError")
except TypeError:
    _ledger.append(1)

# [1,2,3].pop('x')
try:
    _l: Any = [1, 2, 3]
    _ = _l.pop(_str)
    raise AssertionError("list.pop('x') must raise TypeError")
except TypeError:
    _ledger.append(1)

# 'abc'.partition(5) — sep must be str
try:
    _ = "abc".partition(_int)
    raise AssertionError("'abc'.partition(5) must raise TypeError")
except TypeError:
    _ledger.append(1)

# 'abc'.partition(None)
try:
    _ = "abc".partition(_none)
    raise AssertionError("'abc'.partition(None) must raise TypeError")
except TypeError:
    _ledger.append(1)

# 'abc'.rpartition(5) — sep must be str
try:
    _ = "abc".rpartition(_int)
    raise AssertionError("'abc'.rpartition(5) must raise TypeError")
except TypeError:
    _ledger.append(1)

# b'abc'.split(5) — sep must be bytes or None
try:
    _ = b"abc".split(_int)
    raise AssertionError("b'abc'.split(5) must raise TypeError")
except TypeError:
    _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_str_method_width_index_wrong_type_silent {sum(_ledger)} asserts")
