# Spec seed for CPython ZeroDivisionError / ValueError / TypeError
# contract on the arithmetic / constructor argument-validation
# corners that mamba silently returns None / empty values from.
# Surface: CPython rejects (1) `divmod(_, 0)` and `divmod(_, 0.0)`
# because integer/float divide-by-zero is mathematically undefined —
# ZeroDivisionError, not silent None; (2) `pow(_, _, 0)` because the
# 3-argument form is modular exponentiation and the modulus 0 is
# invalid — ValueError, not silent None; (3) `bytes(-1)` /
# `bytearray(-1)` because the byte-buffer pre-fill length must be
# non-negative — ValueError, not silent empty bytes; (4) `bytes(1.5)`
# because float is not a valid bytes constructor source (no
# `__bytes__` and no `__index__`) — TypeError, not silent empty
# bytes. Existing `lang_indexerror_zerodiv_silent` covers integer
# `5 / 0` / `5 // 0` / `5 % 0` and indexerror corners but doesn't
# touch divmod / pow-3rd-arg / bytes-negative / bytes-float.
#
# Probes (every form CPython rejects, mamba silently coerces):
#   • divmod(10, 0)              → mamba: None    (ZeroDivisionError)
#   • divmod(10.0, 0.0)          → mamba: None    (ZeroDivisionError)
#   • divmod(10, 0.0)            → mamba: None    (ZeroDivisionError)
#   • divmod(10.0, 0)            → mamba: None    (ZeroDivisionError)
#   • pow(2, 3, 0)               → mamba: None    (ValueError)
#   • pow(5, 10, 0)              → mamba: None    (ValueError)
#   • bytes(-1)                  → mamba: b''     (ValueError)
#   • bytes(-100)                → mamba: b''     (ValueError)
#   • bytearray(-1)              → mamba: empty   (ValueError)
#   • bytearray(-50)             → mamba: empty   (ValueError)
#   • bytes(1.5)                 → mamba: b''     (TypeError)
#   • bytes(2.5)                 → mamba: b''     (TypeError)
#
# CPython contract:
#   divmod(_, 0)            → ZeroDivisionError("integer division or
#                                  modulo by zero");
#   divmod(_, 0.0)          → ZeroDivisionError("float divmod()");
#   pow(_, _, 0)            → ValueError("pow() 3rd argument cannot
#                                  be 0");
#   bytes(neg) / bytearray(neg)
#                          → ValueError("negative count");
#   bytes(float)            → TypeError("cannot convert 'float'
#                                  object to bytes").
#
# `Any`-typed holders push the operand past static type-checkers
# (Pyright) and past mamba's compile-time argtype enforcement so the
# runtime divergence is what's exercised.
from typing import Any
_ledger: list[int] = []

_zero: Any = 0
_zerof: Any = 0.0
_ten: Any = 10
_tenf: Any = 10.0
_two: Any = 2
_three: Any = 3
_five: Any = 5
_neg1: Any = -1
_neg100: Any = -100
_neg50: Any = -50
_f15: Any = 1.5
_f25: Any = 2.5

# divmod(10, 0)
try:
    _ = divmod(_ten, _zero)
    raise AssertionError("divmod(10, 0) must raise ZeroDivisionError")
except ZeroDivisionError:
    _ledger.append(1)

# divmod(10.0, 0.0)
try:
    _ = divmod(_tenf, _zerof)
    raise AssertionError("divmod(10.0, 0.0) must raise ZeroDivisionError")
except ZeroDivisionError:
    _ledger.append(1)

# divmod(10, 0.0)
try:
    _ = divmod(_ten, _zerof)
    raise AssertionError("divmod(10, 0.0) must raise ZeroDivisionError")
except ZeroDivisionError:
    _ledger.append(1)

# divmod(10.0, 0)
try:
    _ = divmod(_tenf, _zero)
    raise AssertionError("divmod(10.0, 0) must raise ZeroDivisionError")
except ZeroDivisionError:
    _ledger.append(1)

# pow(2, 3, 0) — modular exponentiation with 0 modulus is invalid
try:
    _ = pow(_two, _three, _zero)
    raise AssertionError("pow(2, 3, 0) must raise ValueError")
except ValueError:
    _ledger.append(1)

# pow(5, 10, 0)
try:
    _ = pow(_five, _ten, _zero)
    raise AssertionError("pow(5, 10, 0) must raise ValueError")
except ValueError:
    _ledger.append(1)

# bytes(-1) — negative count
try:
    _ = bytes(_neg1)
    raise AssertionError("bytes(-1) must raise ValueError")
except ValueError:
    _ledger.append(1)

# bytes(-100) — negative count
try:
    _ = bytes(_neg100)
    raise AssertionError("bytes(-100) must raise ValueError")
except ValueError:
    _ledger.append(1)

# bytearray(-1) — negative count
try:
    _ = bytearray(_neg1)
    raise AssertionError("bytearray(-1) must raise ValueError")
except ValueError:
    _ledger.append(1)

# bytearray(-50) — negative count
try:
    _ = bytearray(_neg50)
    raise AssertionError("bytearray(-50) must raise ValueError")
except ValueError:
    _ledger.append(1)

# bytes(1.5) — float not convertible to bytes
try:
    _ = bytes(_f15)
    raise AssertionError("bytes(1.5) must raise TypeError")
except TypeError:
    _ledger.append(1)

# bytes(2.5) — same rule, different value
try:
    _ = bytes(_f25)
    raise AssertionError("bytes(2.5) must raise TypeError")
except TypeError:
    _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_zerodiv_pow_negcount_silent {sum(_ledger)} asserts")
