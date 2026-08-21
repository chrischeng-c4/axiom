# Spec seed for CPython ValueError contract on domain-violating
# builtin / sequence-method calls. Surface: CPython raises ValueError
# on a family of operations where the argument is well-typed but
# outside the supported value domain. Mamba 0.3.60 silently returns
# `None`, `-1`, `b''`, `-0.0`, or the wrong sentinel on every one of
# these forms.
#
# Probes:
#   • chr(-1) — negative codepoint;
#   • chr(0x110000) — codepoint above U+10FFFF;
#   • (1,2,3).index(99) — element not present in tuple;
#   • (1,2,3).index(1, 1) — element exists but not at index >= 1;
#   • min([]) — empty sequence with no `default=` kwarg;
#   • max([]) — empty sequence with no `default=` kwarg;
#   • 1 << -1 — negative shift count;
#   • 1 >> -1 — negative shift count;
#   • bytes.fromhex('ZZ') — non-hexadecimal character;
#   • bytes.fromhex('F') — odd number of hex digits.
#
# CPython contract:
#   chr(out_of_range) → ValueError("chr() arg not in range(0x110000)");
#   tuple.index(missing) → ValueError("tuple.index(x): x not in tuple");
#   min([]) / max([]) → ValueError("min() arg is an empty sequence");
#   1 << -1 → ValueError("negative shift count");
#   bytes.fromhex(invalid) → ValueError("non-hexadecimal number found
#   in fromhex() arg at position N").
#
# `Any`-typed holders bypass static type-checkers (Pyright) and
# mamba's compile-time enforcement so the runtime divergence is what's
# exercised.
from typing import Any
_ledger: list[int] = []

# chr(-1) — negative codepoint
try:
    _ = chr(-1)
    raise AssertionError("chr(-1) must raise ValueError")
except ValueError:
    _ledger.append(1)

# chr(0x110000) — beyond Unicode upper bound
try:
    _ = chr(0x110000)
    raise AssertionError("chr(0x110000) must raise ValueError")
except ValueError:
    _ledger.append(1)

# tuple.index(missing) — element not in tuple
_t: Any = (1, 2, 3)
try:
    _ = _t.index(99)
    raise AssertionError("(1,2,3).index(99) must raise ValueError")
except ValueError:
    _ledger.append(1)

# tuple.index(present, start_idx_past_position) — element exists but
# only at an earlier index than the start
try:
    _ = _t.index(1, 1)
    raise AssertionError("(1,2,3).index(1, 1) must raise ValueError")
except ValueError:
    _ledger.append(1)

# min([]) — empty sequence without default
_empty: Any = []
try:
    _ = min(_empty)
    raise AssertionError("min([]) must raise ValueError")
except ValueError:
    _ledger.append(1)

# max([]) — empty sequence without default
try:
    _ = max(_empty)
    raise AssertionError("max([]) must raise ValueError")
except ValueError:
    _ledger.append(1)

# 1 << -1 — negative shift count
_one: Any = 1
try:
    _ = _one << -1
    raise AssertionError("1 << -1 must raise ValueError")
except ValueError:
    _ledger.append(1)

# 1 >> -1 — negative shift count
try:
    _ = _one >> -1
    raise AssertionError("1 >> -1 must raise ValueError")
except ValueError:
    _ledger.append(1)

# bytes.fromhex('ZZ') — non-hex character
try:
    _ = bytes.fromhex("ZZ")
    raise AssertionError("bytes.fromhex('ZZ') must raise ValueError")
except ValueError:
    _ledger.append(1)

# bytes.fromhex('F') — odd-length hex string
try:
    _ = bytes.fromhex("F")
    raise AssertionError("bytes.fromhex('F') must raise ValueError")
except ValueError:
    _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_valueerror_domain_violation {sum(_ledger)} asserts")
