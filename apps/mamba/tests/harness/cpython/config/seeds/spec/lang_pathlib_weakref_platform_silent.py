# Operational AssertionPass seed for SILENT divergences across the
# filesystem-path / weak-reference / platform-detection /
# locale-helper quad pinned by atomic 172: `pathlib` (the
# documented `Path(p).name` / `.stem` / `.suffix` / `.parent` /
# `.parts` / `str(Path(p))` / `.is_absolute` instance surface),
# `weakref` (the documented `weakref.ref(obj)()` live-deref
# contract), `platform` (the documented `version` /
# `architecture` module-level helper attribute surface), and
# `locale` (the documented `localeconv` module-level helper
# attribute surface).
#
# The matching subset (pathlib class-identifier hasattr surface
# + Path() constructor type contract, fnmatch full value layer
# + module hasattr surface, sysconfig str-return value layer +
# module hasattr surface, platform str-return value layer
# (system / machine / python_version) + partial module hasattr
# surface, locale tuple-return + setlocale / LC constant
# module hasattr surface, time monotonic / perf_counter float-
# return monotonic-progress layer, weakref module hasattr
# surface) is covered by
# `test_pathlib_fnmatch_sysconfig_platform_time_value_ops`;
# this fixture pins the CPython-only contracts that mamba
# currently elides.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • str(Path("/tmp/foo/bar.txt")) == "/tmp/foo/bar.txt" —
#     documented Path __str__ contract (mamba: returns
#     "<PosixPath instance>" — the documented stringification
#     is replaced by a placeholder repr);
#   • Path("/tmp/foo/bar.txt").name == "bar.txt" — documented
#     Path instance attribute (mamba: returns None);
#   • Path("/tmp/foo/bar.txt").stem == "bar" — documented
#     Path instance attribute (mamba: returns None);
#   • Path("/tmp/foo/bar.txt").suffix == ".txt" — documented
#     Path instance attribute (mamba: returns None);
#   • Path("/tmp/foo/bar.txt").parts == ("/", "tmp", "foo",
#     "bar.txt") — documented Path .parts tuple (mamba:
#     returns None);
#   • Path("/tmp").is_absolute() is True — documented Path
#     instance method (mamba: AttributeError 'PosixPath' object
#     has no attribute 'is_absolute');
#   • weakref.ref(obj)() is obj — documented live-deref
#     contract (mamba: returns None — the weak reference
#     can never be dereferenced to the live object);
#   • hasattr(platform, "version") is True — documented
#     module-level helper (mamba: False);
#   • hasattr(platform, "architecture") is True — documented
#     module-level helper (mamba: False);
#   • hasattr(locale, "localeconv") is True — documented
#     module-level helper (mamba: False).
import pathlib as _pathlib_mod
import platform as _platform_mod
import locale as _locale_mod
import weakref as _weakref_mod
from typing import Any

# Module bindings retyped as `Any` to bypass Pyright stub-driven
# narrowing — every spec contract below probes documented public
# instance methods / class attributes / module-level helpers
# that mamba's bundled type stubs do not surface accurately.
pathlib: Any = _pathlib_mod
platform: Any = _platform_mod
locale: Any = _locale_mod
weakref: Any = _weakref_mod


class _C:
    pass


_ledger: list[int] = []

# 1) pathlib.Path — __str__ + name / stem / suffix / parts
_p = pathlib.Path("/tmp/foo/bar.txt")
assert str(_p) == "/tmp/foo/bar.txt"; _ledger.append(1)
assert _p.name == "bar.txt"; _ledger.append(1)
assert _p.stem == "bar"; _ledger.append(1)
assert _p.suffix == ".txt"; _ledger.append(1)
assert _p.parts == ("/", "tmp", "foo", "bar.txt"); _ledger.append(1)

# 2) pathlib.Path — is_absolute instance method
assert pathlib.Path("/tmp").is_absolute() == True; _ledger.append(1)

# 3) weakref.ref — live-deref contract
_obj = _C()
_wr = weakref.ref(_obj)
assert _wr() is _obj; _ledger.append(1)

# 4) platform — version / architecture module-level helper
assert hasattr(platform, "version") == True; _ledger.append(1)
assert hasattr(platform, "architecture") == True; _ledger.append(1)

# 5) locale — localeconv module-level helper
assert hasattr(locale, "localeconv") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_pathlib_weakref_platform_silent {sum(_ledger)} asserts")
