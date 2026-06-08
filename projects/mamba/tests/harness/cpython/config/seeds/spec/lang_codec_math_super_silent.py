# Spec seed for CPython ValueError / LookupError / RuntimeError
# contract on the codec / math-domain / super() corners that mamba
# silently returns numeric / passthrough / placeholder values from.
# Surface: CPython rejects (1) `tuple.index(missing)` because the
# search target is genuinely absent — ValueError, not silent `-1`;
# (2) `str.encode(bad_codec)` / `bytes.decode(bad_codec)` because the
# codec registry has no entry for the named encoding — LookupError,
# not silent passthrough of the raw payload; (3) `super()` invoked
# outside any method body because there is no `__class__` cell to
# bind to — RuntimeError, not a silent dummy super-instance; (4)
# `math.log10(0)` / `math.log2(0)` because the limit of log at 0 is
# -infinity and CPython explicitly checks for the domain edge —
# ValueError, not silent `-inf`; (5) `math.asin(2)` / `math.acos(-2)`
# because |x|>1 is outside the inverse-sine/cosine domain — ValueError,
# not silent `nan`. Existing `lang_*_silent` seeds cover other
# arithmetic / typing corners but none touch this exact mix.
#
# Probes (every form CPython rejects, mamba silently coerces):
#   • (1, 2, 3).index(99)            → mamba: -1       (ValueError)
#   • (1, 2, 3).index(0)             → mamba: -1       (ValueError)
#   • "hi".encode("nonexistent_xxx") → mamba: b'hi'    (LookupError)
#   • b"hi".decode("nonexistent_xxx")→ mamba: 'hi'     (LookupError)
#   • super() (outside class)        → mamba: <super>  (RuntimeError)
#   • math.log10(0.0)                → mamba: -inf     (ValueError)
#   • math.log2(0.0)                 → mamba: -inf     (ValueError)
#   • math.asin(2.0)                 → mamba: nan      (ValueError)
#   • math.asin(-2.0)                → mamba: nan      (ValueError)
#   • math.acos(2.0)                 → mamba: nan      (ValueError)
#   • math.acos(-2.0)                → mamba: nan      (ValueError)
#
# CPython contract:
#   tuple.index(absent)     → ValueError("tuple.index(x): x not in
#                                  tuple");
#   str.encode(bad_codec)
#   bytes.decode(bad_codec) → LookupError("unknown encoding: …");
#   super() outside method  → RuntimeError("super(): no arguments");
#   math.log10(0) / log2(0) → ValueError("math domain error");
#   math.asin(|x|>1)
#   math.acos(|x|>1)        → ValueError("math domain error").
#
# `Any`-typed holders push the operand past static type-checkers
# (Pyright) and past mamba's compile-time argtype enforcement so the
# runtime divergence is what's exercised.
from typing import Any
import math
_ledger: list[int] = []

_tup: Any = (1, 2, 3)
_str: Any = "hi"
_b: Any = b"hi"
_bad_codec: Any = "nonexistent_codec_xxx_42"
_f0: Any = 0.0
_f2: Any = 2.0
_fneg2: Any = -2.0

# tuple.index(99) — value genuinely absent
try:
    _ = _tup.index(99)
    raise AssertionError("(1,2,3).index(99) must raise ValueError")
except ValueError:
    _ledger.append(1)

# tuple.index(0) — also absent
try:
    _ = _tup.index(0)
    raise AssertionError("(1,2,3).index(0) must raise ValueError")
except ValueError:
    _ledger.append(1)

# "hi".encode("nonexistent_codec_xxx_42") — codec registry miss
try:
    _ = _str.encode(_bad_codec)
    raise AssertionError("str.encode(bad_codec) must raise LookupError")
except LookupError:
    _ledger.append(1)

# b"hi".decode("nonexistent_codec_xxx_42") — codec registry miss
try:
    _ = _b.decode(_bad_codec)
    raise AssertionError("bytes.decode(bad_codec) must raise LookupError")
except LookupError:
    _ledger.append(1)

# super() outside a class body — no __class__ cell to bind
try:
    _ = super()
    raise AssertionError("super() outside class must raise RuntimeError")
except RuntimeError:
    _ledger.append(1)

# math.log10(0.0) — limit is -inf, CPython rejects the domain edge
try:
    _ = math.log10(_f0)
    raise AssertionError("math.log10(0) must raise ValueError")
except ValueError:
    _ledger.append(1)

# math.log2(0.0)
try:
    _ = math.log2(_f0)
    raise AssertionError("math.log2(0) must raise ValueError")
except ValueError:
    _ledger.append(1)

# math.asin(2.0) — |x|>1, outside domain
try:
    _ = math.asin(_f2)
    raise AssertionError("math.asin(2.0) must raise ValueError")
except ValueError:
    _ledger.append(1)

# math.asin(-2.0)
try:
    _ = math.asin(_fneg2)
    raise AssertionError("math.asin(-2.0) must raise ValueError")
except ValueError:
    _ledger.append(1)

# math.acos(2.0)
try:
    _ = math.acos(_f2)
    raise AssertionError("math.acos(2.0) must raise ValueError")
except ValueError:
    _ledger.append(1)

# math.acos(-2.0)
try:
    _ = math.acos(_fneg2)
    raise AssertionError("math.acos(-2.0) must raise ValueError")
except ValueError:
    _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_codec_math_super_silent {sum(_ledger)} asserts")
