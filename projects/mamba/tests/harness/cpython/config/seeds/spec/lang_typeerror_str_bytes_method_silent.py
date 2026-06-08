# Spec seed for CPython TypeError contract on string / bytes / int /
# `sorted(key=)` method-arg type checks. Surface: CPython rejects every
# form below with TypeError; mamba 0.3.60 silently returns `''` /
# `b''` / `False` / `-1` / `None` / `0` / a numeric result instead of
# dispatching the type-protocol → TypeError fallback.
# Existing lang_typeerror_* seeds cover binary arithmetic / call-arity
# / unhashable / iter-required / immutable-mutation / ordering /
# numeric-conversion-constructor / bitwise-unary / seq-repeat-builtin
# angles; this seed adds the
# str/bytes-method-arg-must-be-str/bytes-not-int family plus
# `int(str, str_base)` and `sorted(key=non_callable)` corners.
#
# Probes (every form CPython raises TypeError on, mamba silently
# returns a wrong-shape value):
#   • ''.join([1, 2])         → mamba: '' (join silently drops)
#   • ''.join([b'a', b'b'])   → mamba: ''
#   • b''.join(['a', 'b'])    → mamba: b''
#   • b''.join([1, 2])        → mamba: b''
#   • 'abc'.startswith(1)     → mamba: False
#   • 'abc'.endswith(1)       → mamba: False
#   • 'abc'.startswith([1])   → mamba: False (list not tuple)
#   • 'abc'.find(1)           → mamba: -1
#   • 'abc'.replace(1, 'b')   → mamba: None
#   • int('1', '10')          → mamba: 1 (base coerced silently)
#   • 'abc'.count(1)          → mamba: 0
#   • 'a,b,c'.split(1)        → mamba: None
#   • sorted([3,1,2], key=5)  → mamba: [1, 2, 3] (key is non-callable
#                               int; mamba ignores key entirely).
#
# CPython contract:
#   ''.join(non_str_iter)     → TypeError("sequence item 0: expected
#                                   str instance, <typename> found");
#   b''.join(non_bytes_iter)  → TypeError("sequence item 0: expected
#                                   a bytes-like object, <typename>
#                                   found");
#   str.startswith(non_str)   → TypeError("startswith first arg must
#                                   be str or a tuple of str, not
#                                   <typename>");
#   str.find(non_str)         → TypeError("must be str, not
#                                   <typename>");
#   str.replace(non_str, ...) → TypeError("replace() argument 1
#                                   must be str, not <typename>");
#   int('1', non_int_base)    → TypeError("'str' object cannot be
#                                   interpreted as an integer");
#   str.count(non_str)        → TypeError("must be str, not
#                                   <typename>");
#   str.split(non_str)        → TypeError("must be str or None, not
#                                   <typename>");
#   sorted(_, key=non_call)   → TypeError("'<typename>' object is
#                                   not callable").
#
# `Any`-typed holders push the operand past static type-checkers
# (Pyright) and past mamba's compile-time argtype enforcement so the
# runtime divergence is what's exercised.
from typing import Any
_ledger: list[int] = []

_lst_int: Any = [1, 2]
_lst_bytes: Any = [b'a', b'b']
_lst_str: Any = ['a', 'b']
_one: Any = 1
_list1: Any = [1]
_b_str: Any = 'b'
_ten_str: Any = '10'
_five: Any = 5

# ''.join([1, 2]) — str.join requires every element to be str
try:
    _ = ''.join(_lst_int)
    raise AssertionError("''.join([1, 2]) must raise TypeError")
except TypeError:
    _ledger.append(1)

# ''.join([b'a', b'b']) — str.join rejects bytes elements
try:
    _ = ''.join(_lst_bytes)
    raise AssertionError("''.join([b'a', b'b']) must raise TypeError")
except TypeError:
    _ledger.append(1)

# b''.join(['a', 'b']) — bytes.join requires bytes-like elements
try:
    _ = b''.join(_lst_str)
    raise AssertionError("b''.join(['a', 'b']) must raise TypeError")
except TypeError:
    _ledger.append(1)

# b''.join([1, 2]) — bytes.join rejects int elements
try:
    _ = b''.join(_lst_int)
    raise AssertionError("b''.join([1, 2]) must raise TypeError")
except TypeError:
    _ledger.append(1)

# 'abc'.startswith(1) — startswith requires str or tuple of str
try:
    _ = 'abc'.startswith(_one)
    raise AssertionError("'abc'.startswith(1) must raise TypeError")
except TypeError:
    _ledger.append(1)

# 'abc'.endswith(1) — endswith requires str or tuple of str
try:
    _ = 'abc'.endswith(_one)
    raise AssertionError("'abc'.endswith(1) must raise TypeError")
except TypeError:
    _ledger.append(1)

# 'abc'.startswith([1]) — list (not tuple) of str-or-not is rejected
try:
    _ = 'abc'.startswith(_list1)
    raise AssertionError("'abc'.startswith([1]) must raise TypeError")
except TypeError:
    _ledger.append(1)

# 'abc'.find(1) — find requires str argument
try:
    _ = 'abc'.find(_one)
    raise AssertionError("'abc'.find(1) must raise TypeError")
except TypeError:
    _ledger.append(1)

# 'abc'.replace(1, 'b') — replace requires str arg 1
try:
    _ = 'abc'.replace(_one, _b_str)
    raise AssertionError("'abc'.replace(1, 'b') must raise TypeError")
except TypeError:
    _ledger.append(1)

# int('1', '10') — base must be int, not str
try:
    _ = int('1', _ten_str)
    raise AssertionError("int('1', '10') must raise TypeError")
except TypeError:
    _ledger.append(1)

# 'abc'.count(1) — count requires str argument
try:
    _ = 'abc'.count(_one)
    raise AssertionError("'abc'.count(1) must raise TypeError")
except TypeError:
    _ledger.append(1)

# 'a,b,c'.split(1) — split requires str or None separator
try:
    _ = 'a,b,c'.split(_one)
    raise AssertionError("'a,b,c'.split(1) must raise TypeError")
except TypeError:
    _ledger.append(1)

# sorted([3, 1, 2], key=5) — key must be a callable
try:
    _ = sorted([3, 1, 2], key=_five)
    raise AssertionError("sorted(_, key=5) must raise TypeError")
except TypeError:
    _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_typeerror_str_bytes_method_silent {sum(_ledger)} asserts")
