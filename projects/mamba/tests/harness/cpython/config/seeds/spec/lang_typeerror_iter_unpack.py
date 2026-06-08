# Spec seed for CPython TypeError contract on iteration / unpacking
# / format dispatch.
# Surface: CPython raises TypeError when:
#   • the RHS of a tuple-unpack `a, b = X` is not iterable;
#   • a `for x in X:` loop runs against a non-iterable X;
#   • `*X` star-unpack inside a list/tuple literal targets a
#     non-iterable;
#   • `**X` dict-spread inside a dict literal targets a non-mapping;
#   • the `%` string-formatting operator gets a wrong-type
#     argument for the format-spec ("%d" % "abc"),
#     ("%f" % "abc"),
#     ("%c" % 1.5);
#   • a function call passes the wrong number of arguments to a
#     fixed-arity callable (TypeError-only; not arity-checked here).
#
# Mamba 0.3.60 currently diverges on every form below:
#   • `a, b = int` raises ValueError ("unpack count mismatch")
#     instead of TypeError;
#   • `for x in int:` raises ValueError ("unpack count mismatch")
#     too — same dispatch path;
#   • the rest silently pass / coerce.
# Either divergence still satisfies the spec/ Fail contract — the
# raised exception is not TypeError, so the AssertionError fires.
#
# `Any`-typed holders keep static type-checkers (Pyright) from
# flagging the intentional protocol mismatches before runtime.
from typing import Any
_ledger: list[int] = []

_i: Any = 5
_n: Any = None
_b: Any = True

# Unpack non-iterable int into 2 names
try:
    _a, _b2 = _i  # type: ignore[misc]
    raise AssertionError("`a, b = int` must raise TypeError")
except TypeError:
    _ledger.append(1)

# Unpack non-iterable None
try:
    _a, _b2 = _n  # type: ignore[misc]
    raise AssertionError("`a, b = None` must raise TypeError")
except TypeError:
    _ledger.append(1)

# Unpack non-iterable bool
try:
    _a, _b2 = _b  # type: ignore[misc]
    raise AssertionError("`a, b = bool` must raise TypeError")
except TypeError:
    _ledger.append(1)

# for-loop over non-iterable int
try:
    for _v in _i:  # type: ignore[union-attr]
        pass
    raise AssertionError("`for x in int` must raise TypeError")
except TypeError:
    _ledger.append(1)

# for-loop over non-iterable None
try:
    for _v in _n:  # type: ignore[union-attr]
        pass
    raise AssertionError("`for x in None` must raise TypeError")
except TypeError:
    _ledger.append(1)

# * star-unpack of non-iterable into list literal
try:
    _ = [*_i]  # type: ignore[misc]
    raise AssertionError("`[*int]` must raise TypeError")
except TypeError:
    _ledger.append(1)

# * star-unpack of non-iterable into tuple literal
try:
    _ = (*_i,)  # type: ignore[misc]
    raise AssertionError("`(*int,)` must raise TypeError")
except TypeError:
    _ledger.append(1)

# ** dict-spread of non-mapping
try:
    _ = {**_i}  # type: ignore[dict-item]
    raise AssertionError("`{**int}` must raise TypeError")
except TypeError:
    _ledger.append(1)

# ** dict-spread of None
try:
    _ = {**_n}  # type: ignore[dict-item]
    raise AssertionError("`{**None}` must raise TypeError")
except TypeError:
    _ledger.append(1)

# "%d" % str — format spec d requires int
_s: Any = "abc"
try:
    _ = "%d" % _s
    raise AssertionError("`'%d' %% str` must raise TypeError")
except TypeError:
    _ledger.append(1)

# "%f" % str — format spec f requires float
try:
    _ = "%f" % _s
    raise AssertionError("`'%f' %% str` must raise TypeError")
except TypeError:
    _ledger.append(1)

# "%c" % float — format spec c requires int or single-char str
_f: Any = 1.5
try:
    _ = "%c" % _f
    raise AssertionError("`'%c' %% float` must raise TypeError")
except TypeError:
    _ledger.append(1)

# next() on non-iterator (sequence is iterable but not an iterator)
_lst: Any = [1, 2, 3]
try:
    _ = next(_lst)
    raise AssertionError("`next(list)` must raise TypeError (list is iterable, not iterator)")
except TypeError:
    _ledger.append(1)

# next() on int (non-iterator)
try:
    _ = next(_i)
    raise AssertionError("`next(int)` must raise TypeError")
except TypeError:
    _ledger.append(1)

# sorted() with non-callable key
try:
    _ = sorted([1, 2, 3], key=_i)
    raise AssertionError("`sorted(..., key=int)` must raise TypeError")
except TypeError:
    _ledger.append(1)

# max() with no args
try:
    _ = max()  # type: ignore[call-overload]
    raise AssertionError("`max()` must raise TypeError")
except TypeError:
    _ledger.append(1)

# min() with no args
try:
    _ = min()  # type: ignore[call-overload]
    raise AssertionError("`min()` must raise TypeError")
except TypeError:
    _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_typeerror_iter_unpack {sum(_ledger)} asserts")
