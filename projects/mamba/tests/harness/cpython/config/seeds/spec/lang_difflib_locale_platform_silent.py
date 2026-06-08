# Operational AssertionPass seed for SILENT divergences across the
# difflib extended module-helper surface + difflib.SequenceMatcher
# class-identity / instance .ratio value contract + difflib.
# unified_diff list-emission contract + linecache extended module-
# helper surface + locale extended module-helper surface + getpass
# extended module-helper surface + platform extended module-helper
# surface pinned by atomic 185: `difflib` (the documented `Differ`
# / `HtmlDiff` / `ndiff` / `context_diff` / `restore` /
# `IS_LINE_JUNK` / `IS_CHARACTER_JUNK` class / function / sentinel
# identifiers + the documented `type(SequenceMatcher(None, a, b))
# .__name__ == "SequenceMatcher"` class-identity contract + the
# documented `SequenceMatcher.ratio()` value contract + the
# documented `unified_diff` list-emission contract), `linecache`
# (the documented `cache` / `lazycache` extended class / function
# identifiers), `locale` (the documented `getdefaultlocale` /
# `LC_COLLATE` / `LC_MONETARY` / `LC_MESSAGES` / `Error` /
# `localeconv` extended class / function / exception
# identifiers), `getpass` (the documented `GetPassWarning`
# exception identifier), and `platform` (the documented
# `version` / `python_implementation` / `python_compiler` /
# `python_build` / `python_branch` / `architecture` / `uname`
# extended function identifiers).
#
# The matching subset (partial difflib module hasattr surface
# (SequenceMatcher / unified_diff / get_close_matches) +
# difflib.get_close_matches + full fnmatch module hasattr
# surface + fnmatch value contracts + partial linecache module
# hasattr surface (getline / getlines / checkcache /
# clearcache) + full mimetypes module hasattr surface +
# mimetypes guess_type/guess_extension + partial locale module
# hasattr surface (getlocale / setlocale / LC_ALL / LC_CTYPE /
# LC_NUMERIC / LC_TIME) + partial getpass module hasattr
# surface (getuser / getpass) + getpass.getuser str return
# type + partial platform module hasattr surface (system /
# node / release / machine / processor / platform /
# python_version) + platform str return types) is covered by
# `test_difflib_fnmatch_mimetypes_locale_getpass_platform_hasattr_ops`;
# this fixture pins the CPython-only contracts that mamba
# currently elides.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • hasattr(difflib, "Differ") is True — documented class
#     identifier (mamba: False);
#   • hasattr(difflib, "HtmlDiff") is True — documented
#     class identifier (mamba: False);
#   • hasattr(difflib, "ndiff") is True — documented
#     function identifier (mamba: False);
#   • hasattr(difflib, "context_diff") is True — documented
#     function identifier (mamba: False);
#   • hasattr(difflib, "restore") is True — documented
#     function identifier (mamba: False);
#   • hasattr(difflib, "IS_LINE_JUNK") is True — documented
#     sentinel identifier (mamba: False);
#   • hasattr(difflib, "IS_CHARACTER_JUNK") is True —
#     documented sentinel identifier (mamba: False);
#   • type(difflib.SequenceMatcher(None, "abcd", "abcf"))
#     .__name__ == "SequenceMatcher" — documented class-
#     identity contract (mamba: returns "float" — the
#     constructor short-circuits to the .ratio() float
#     directly);
#   • difflib.SequenceMatcher(None, "abcd", "abcf").ratio()
#     == 0.75 — documented instance-method value contract
#     (mamba: raises AttributeError on .ratio because the
#     instance is already a float);
#   • len(list(difflib.unified_diff(...))) > 0 — documented
#     list-emission contract (mamba: returns empty list);
#   • hasattr(linecache, "cache") is True — documented
#     module-level sentinel (mamba: False);
#   • hasattr(linecache, "lazycache") is True — documented
#     function identifier (mamba: False);
#   • hasattr(locale, "getdefaultlocale") is True —
#     documented function identifier (mamba: False);
#   • hasattr(locale, "LC_COLLATE") is True — documented
#     integer sentinel (mamba: False);
#   • hasattr(locale, "LC_MONETARY") is True — documented
#     integer sentinel (mamba: False);
#   • hasattr(locale, "LC_MESSAGES") is True — documented
#     integer sentinel (mamba: False);
#   • hasattr(locale, "Error") is True — documented
#     exception identifier (mamba: False);
#   • hasattr(locale, "localeconv") is True — documented
#     function identifier (mamba: False);
#   • hasattr(getpass, "GetPassWarning") is True —
#     documented exception identifier (mamba: False);
#   • hasattr(platform, "version") is True — documented
#     function identifier (mamba: False);
#   • hasattr(platform, "python_implementation") is True —
#     documented function identifier (mamba: False);
#   • hasattr(platform, "python_compiler") is True —
#     documented function identifier (mamba: False);
#   • hasattr(platform, "python_build") is True —
#     documented function identifier (mamba: False);
#   • hasattr(platform, "python_branch") is True —
#     documented function identifier (mamba: False);
#   • hasattr(platform, "architecture") is True —
#     documented function identifier (mamba: False);
#   • hasattr(platform, "uname") is True — documented
#     function identifier (mamba: False).
import difflib as _difflib_mod
import linecache as _linecache_mod
import locale as _locale_mod
import getpass as _getpass_mod
import platform as _platform_mod
from typing import Any

# Module bindings retyped as `Any` to bypass Pyright stub-driven
# narrowing — every spec contract below probes documented public
# class identifiers / instance-method / value-contract behavior
# that mamba's bundled type stubs do not surface accurately.
difflib: Any = _difflib_mod
linecache: Any = _linecache_mod
locale: Any = _locale_mod
getpass: Any = _getpass_mod
platform: Any = _platform_mod


_ledger: list[int] = []

# 1) difflib — extended module-helper surface
assert hasattr(difflib, "Differ") == True; _ledger.append(1)
assert hasattr(difflib, "HtmlDiff") == True; _ledger.append(1)
assert hasattr(difflib, "ndiff") == True; _ledger.append(1)
assert hasattr(difflib, "context_diff") == True; _ledger.append(1)
assert hasattr(difflib, "restore") == True; _ledger.append(1)
assert hasattr(difflib, "IS_LINE_JUNK") == True; _ledger.append(1)
assert hasattr(difflib, "IS_CHARACTER_JUNK") == True; _ledger.append(1)

# 2) difflib.SequenceMatcher — class identity + ratio value
_sm = difflib.SequenceMatcher(None, "abcd", "abcf")
assert type(_sm).__name__ == "SequenceMatcher"; _ledger.append(1)
assert _sm.ratio() == 0.75; _ledger.append(1)

# 3) difflib.unified_diff — list-emission contract
_diff = list(difflib.unified_diff(["a", "b", "c"], ["a", "x", "c"], lineterm=""))
assert len(_diff) > 0; _ledger.append(1)

# 4) linecache — extended module-helper surface
assert hasattr(linecache, "cache") == True; _ledger.append(1)
assert hasattr(linecache, "lazycache") == True; _ledger.append(1)

# 5) locale — extended module-helper surface
assert hasattr(locale, "getdefaultlocale") == True; _ledger.append(1)
assert hasattr(locale, "LC_COLLATE") == True; _ledger.append(1)
assert hasattr(locale, "LC_MONETARY") == True; _ledger.append(1)
assert hasattr(locale, "LC_MESSAGES") == True; _ledger.append(1)
assert hasattr(locale, "Error") == True; _ledger.append(1)
assert hasattr(locale, "localeconv") == True; _ledger.append(1)

# 6) getpass — extended exception identifier
assert hasattr(getpass, "GetPassWarning") == True; _ledger.append(1)

# 7) platform — extended function identifiers
assert hasattr(platform, "version") == True; _ledger.append(1)
assert hasattr(platform, "python_implementation") == True; _ledger.append(1)
assert hasattr(platform, "python_compiler") == True; _ledger.append(1)
assert hasattr(platform, "python_build") == True; _ledger.append(1)
assert hasattr(platform, "python_branch") == True; _ledger.append(1)
assert hasattr(platform, "architecture") == True; _ledger.append(1)
assert hasattr(platform, "uname") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_difflib_locale_platform_silent {sum(_ledger)} asserts")
