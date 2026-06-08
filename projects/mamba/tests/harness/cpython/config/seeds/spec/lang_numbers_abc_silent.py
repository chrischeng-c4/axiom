# Operational AssertionPass seed for SILENT divergences across
# the `numbers` ABC isinstance / issubclass / class-name /
# MRO surface pinned by atomic 203: `numbers` (the documented
# ABC-registration contract — `isinstance(1, numbers.Number)`
# / `isinstance(1.5, numbers.Real)` /
# `isinstance(1, numbers.Integral)` /
# `isinstance(1+2j, numbers.Complex)` and `issubclass(int,
# numbers.Real)` / `issubclass(int, numbers.Integral)` /
# `issubclass(int, numbers.Number)` / `issubclass(float,
# numbers.Real)` — mamba returns False everywhere — and
# the documented class-name attribute surface —
# `numbers.Number.__name__ == "Number"` /
# `numbers.Real.__name__ == "Real"` /
# `numbers.Integral.__name__ == "Integral"` — mamba
# collapses to None — and the documented MRO length —
# `len(numbers.Real.__mro__) == 4` — mamba collapses to 0).
#
# The matching subset (full numbers hasattr, full
# colorsys hasattr + rgb_to_hls / hls_to_rgb round-trip,
# partial mimetypes hasattr + guess_type lookup
# round-trip, full secrets hasattr + token / compare_digest
# round-trip, full signal hasattr + POSIX integer-value
# contract) is covered by
# `test_numbers_colorsys_mimetypes_secrets_signal_value_ops`;
# this fixture pins the CPython-only contracts that mamba
# currently elides.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • isinstance(1, numbers.Number) is True — documented
#     ABC registration (mamba: False);
#   • isinstance(1.5, numbers.Real) is True — documented
#     ABC registration (mamba: False);
#   • isinstance(1, numbers.Integral) is True —
#     documented ABC registration (mamba: False);
#   • isinstance(1 + 2j, numbers.Complex) is True —
#     documented ABC registration (mamba: False);
#   • issubclass(int, numbers.Real) is True — documented
#     ABC registration (mamba: False);
#   • issubclass(int, numbers.Integral) is True —
#     documented ABC registration (mamba: False);
#   • issubclass(int, numbers.Number) is True —
#     documented ABC registration (mamba: False);
#   • issubclass(float, numbers.Real) is True —
#     documented ABC registration (mamba: False);
#   • numbers.Number.__name__ == "Number" — documented
#     class-name attribute (mamba: None);
#   • numbers.Real.__name__ == "Real" — documented
#     class-name attribute (mamba: None);
#   • numbers.Integral.__name__ == "Integral" —
#     documented class-name attribute (mamba: None);
#   • len(numbers.Real.__mro__) == 4 — documented MRO
#     length (mamba: 0).
import numbers as _numbers_mod
from typing import Any

# Module binding retyped as `Any` to bypass Pyright stub-driven
# narrowing — every spec contract below probes documented public
# class identity / module-attribute identifier behavior that
# mamba's bundled type stubs do not surface accurately.
numbers: Any = _numbers_mod


_ledger: list[int] = []

# 1) numbers — ABC isinstance registration contract
assert isinstance(1, numbers.Number) == True; _ledger.append(1)
assert isinstance(1.5, numbers.Real) == True; _ledger.append(1)
assert isinstance(1, numbers.Integral) == True; _ledger.append(1)
assert isinstance(1 + 2j, numbers.Complex) == True; _ledger.append(1)

# 2) numbers — ABC issubclass registration contract
assert issubclass(int, numbers.Real) == True; _ledger.append(1)
assert issubclass(int, numbers.Integral) == True; _ledger.append(1)
assert issubclass(int, numbers.Number) == True; _ledger.append(1)
assert issubclass(float, numbers.Real) == True; _ledger.append(1)

# 3) numbers — class-name attribute contract
assert numbers.Number.__name__ == "Number"; _ledger.append(1)
assert numbers.Real.__name__ == "Real"; _ledger.append(1)
assert numbers.Integral.__name__ == "Integral"; _ledger.append(1)

# 4) numbers — MRO length contract
assert len(numbers.Real.__mro__) == 4; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_numbers_abc_silent {sum(_ledger)} asserts")
