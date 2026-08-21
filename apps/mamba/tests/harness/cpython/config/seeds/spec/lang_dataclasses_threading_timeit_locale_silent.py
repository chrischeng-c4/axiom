# Operational AssertionPass seed for SILENT divergences across
# the `dataclasses` module identifier surface +
# `threading.Lock` instance class identity contract +
# `timeit` module identifier surface +
# `timeit.default_timer` return-type contract + `locale`
# module identifier surface pinned by atomic 204:
# `dataclasses` (the documented class / decorator / sentinel
# identifier surface — `replace` / `Field` /
# `FrozenInstanceError` / `InitVar` / `KW_ONLY` / `MISSING`
# / `make_dataclass` / `is_dataclass`),
# `threading.Lock` (the documented
# `type(threading.Lock()).__name__ == "lock"` lowercase
# class-identity contract — mamba collapses to "Lock"),
# `timeit` (the documented `default_repeat` sentinel
# identifier — mamba: False — and the documented
# `type(timeit.default_timer()).__name__ == "float"`
# return-type contract — mamba returns "int"),
# and `locale` (the documented helper / constant
# identifier surface — `getdefaultlocale` /
# `getpreferredencoding` / `LC_COLLATE` / `LC_MONETARY` /
# `LC_MESSAGES` / `Error` / `localeconv` / `atoi` / `atof`
# / `currency` / `str` / `delocalize` / `normalize`).
#
# The matching subset (partial dataclasses hasattr,
# partial asyncio hasattr, full threading hasattr +
# RLock / Event instance class identity, partial timeit
# hasattr, partial locale hasattr + LC_ALL constant-type
# contract) is covered by
# `test_dataclasses_asyncio_threading_timeit_locale_value_ops`;
# this fixture pins the CPython-only contracts that mamba
# currently elides.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • hasattr(dataclasses, "replace") is True —
#     documented helper identifier (mamba: False);
#   • hasattr(dataclasses, "Field") is True — documented
#     class identifier (mamba: False);
#   • hasattr(dataclasses, "FrozenInstanceError") is True
#     — documented exception identifier (mamba: False);
#   • hasattr(dataclasses, "InitVar") is True —
#     documented class identifier (mamba: False);
#   • hasattr(dataclasses, "KW_ONLY") is True —
#     documented sentinel identifier (mamba: False);
#   • hasattr(dataclasses, "MISSING") is True —
#     documented sentinel identifier (mamba: False);
#   • hasattr(dataclasses, "make_dataclass") is True —
#     documented helper identifier (mamba: False);
#   • hasattr(dataclasses, "is_dataclass") is True —
#     documented helper identifier (mamba: False);
#   • type(threading.Lock()).__name__ == "lock" —
#     documented lowercase class-identity
#     (mamba: "Lock");
#   • hasattr(timeit, "default_repeat") is True —
#     documented sentinel identifier (mamba: False);
#   • type(timeit.default_timer()).__name__ == "float"
#     — documented return-type contract (mamba: "int");
#   • hasattr(locale, "getdefaultlocale") is True —
#     documented helper identifier (mamba: False);
#   • hasattr(locale, "getpreferredencoding") is True —
#     documented helper identifier (mamba: False);
#   • hasattr(locale, "LC_COLLATE") is True —
#     documented constant identifier (mamba: False);
#   • hasattr(locale, "LC_MONETARY") is True —
#     documented constant identifier (mamba: False);
#   • hasattr(locale, "LC_MESSAGES") is True —
#     documented constant identifier (mamba: False);
#   • hasattr(locale, "Error") is True — documented
#     exception identifier (mamba: False);
#   • hasattr(locale, "localeconv") is True —
#     documented helper identifier (mamba: False);
#   • hasattr(locale, "atoi") is True — documented
#     helper identifier (mamba: False);
#   • hasattr(locale, "atof") is True — documented
#     helper identifier (mamba: False);
#   • hasattr(locale, "currency") is True — documented
#     helper identifier (mamba: False);
#   • hasattr(locale, "str") is True — documented
#     helper identifier (mamba: False);
#   • hasattr(locale, "delocalize") is True —
#     documented helper identifier (mamba: False);
#   • hasattr(locale, "normalize") is True —
#     documented helper identifier (mamba: False).
import dataclasses as _dataclasses_mod
import threading as _threading_mod
import timeit as _timeit_mod
import locale as _locale_mod
from typing import Any

# Module bindings retyped as `Any` to bypass Pyright stub-driven
# narrowing — every spec contract below probes documented public
# class identity / module-attribute identifier behavior that
# mamba's bundled type stubs do not surface accurately.
dataclasses: Any = _dataclasses_mod
threading: Any = _threading_mod
timeit: Any = _timeit_mod
locale: Any = _locale_mod


_ledger: list[int] = []

# 1) dataclasses — module identifier surface
assert hasattr(dataclasses, "replace") == True; _ledger.append(1)
assert hasattr(dataclasses, "Field") == True; _ledger.append(1)
assert hasattr(dataclasses, "FrozenInstanceError") == True; _ledger.append(1)
assert hasattr(dataclasses, "InitVar") == True; _ledger.append(1)
assert hasattr(dataclasses, "KW_ONLY") == True; _ledger.append(1)
assert hasattr(dataclasses, "MISSING") == True; _ledger.append(1)
assert hasattr(dataclasses, "make_dataclass") == True; _ledger.append(1)
assert hasattr(dataclasses, "is_dataclass") == True; _ledger.append(1)

# 2) threading.Lock — lowercase class-identity contract
assert type(threading.Lock()).__name__ == "lock"; _ledger.append(1)

# 3) timeit — sentinel identifier + return-type contract
assert hasattr(timeit, "default_repeat") == True; _ledger.append(1)
assert type(timeit.default_timer()).__name__ == "float"; _ledger.append(1)

# 4) locale — module identifier surface
assert hasattr(locale, "getdefaultlocale") == True; _ledger.append(1)
assert hasattr(locale, "getpreferredencoding") == True; _ledger.append(1)
assert hasattr(locale, "LC_COLLATE") == True; _ledger.append(1)
assert hasattr(locale, "LC_MONETARY") == True; _ledger.append(1)
assert hasattr(locale, "LC_MESSAGES") == True; _ledger.append(1)
assert hasattr(locale, "Error") == True; _ledger.append(1)
assert hasattr(locale, "localeconv") == True; _ledger.append(1)
assert hasattr(locale, "atoi") == True; _ledger.append(1)
assert hasattr(locale, "atof") == True; _ledger.append(1)
assert hasattr(locale, "currency") == True; _ledger.append(1)
assert hasattr(locale, "str") == True; _ledger.append(1)
assert hasattr(locale, "delocalize") == True; _ledger.append(1)
assert hasattr(locale, "normalize") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_dataclasses_threading_timeit_locale_silent {sum(_ledger)} asserts")
