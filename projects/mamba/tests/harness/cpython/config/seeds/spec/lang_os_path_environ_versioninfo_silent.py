# Operational AssertionPass seed for SILENT divergences across the
# `os.path` partial-surface + `os.environ` instance class + the
# documented `os.chdir` / `os.putenv` mutation helpers + the
# `sys.version_info` namedtuple subscript / class identity
# surface pinned by atomic 179: `os.path` (the documented
# `isabs` / `normpath` / `expandvars` / `relpath` /
# `commonpath` / `commonprefix` module-level helper hasattr
# surface), `os` (the documented `chdir` / `putenv` mutation
# helper hasattr surface + the documented `environ` instance
# class identity / mapping protocol surface), and `sys` (the
# documented `version_info` namedtuple subscript + class
# identity surface).
#
# The matching subset (os.getcwd / name / sep / linesep
# constant layer, os.path.join / split / splitext / dirname
# / basename string-transform layer, partial os.path hasattr
# surface (exists / isfile / isdir / abspath / expanduser /
# getsize / join / split / splitext / dirname / basename),
# partial os module hasattr surface (getcwd / listdir / mkdir
# / rmdir / remove / rename / stat / environ / name / sep /
# linesep / path / getenv), sys.platform / maxsize /
# byteorder value layer, sys.version_info.major attribute-
# access layer, full sys module hasattr surface, tempfile.
# gettempdir / gettempprefix + full tempfile hasattr surface,
# glob.glob list-return + full glob hasattr surface) is
# covered by `test_os_sys_tempfile_glob_value_ops`; this
# fixture pins the CPython-only contracts that mamba
# currently elides.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • hasattr(os.path, "isabs") is True — documented module-
#     level helper (mamba: False);
#   • hasattr(os.path, "normpath") is True — documented
#     module-level helper (mamba: False);
#   • hasattr(os.path, "expandvars") is True — documented
#     module-level helper (mamba: False);
#   • hasattr(os.path, "relpath") is True — documented
#     module-level helper (mamba: False);
#   • hasattr(os.path, "commonpath") is True — documented
#     module-level helper (mamba: False);
#   • hasattr(os.path, "commonprefix") is True — documented
#     module-level helper (mamba: False);
#   • hasattr(os, "chdir") is True — documented mutation
#     helper (mamba: False);
#   • hasattr(os, "putenv") is True — documented mutation
#     helper (mamba: False);
#   • type(os.environ).__name__ == "_Environ" — documented
#     environ instance class identity (mamba: returns "dict"
#     — the documented Environ instance is replaced by a
#     plain dict);
#   • "PATH" in os.environ is True — documented mapping
#     protocol contract (mamba: False — the documented PATH
#     key is missing from the environ dict);
#   • os.environ.get("PATH", "MISSING") is not "MISSING" —
#     documented mapping protocol get-with-default contract
#     (mamba: returns "MISSING" — the documented PATH key
#     is missing from the environ dict);
#   • type(sys.version_info).__name__ == "version_info" —
#     documented namedtuple class identity (mamba: returns
#     "dict" — the version_info instance is a plain dict
#     not the documented version_info namedtuple);
#   • sys.version_info[0] == 3 — documented namedtuple
#     subscript-access contract (mamba: KeyError 0 — the
#     dict instance does not respond to integer subscript).
import os as _os_mod
import sys as _sys_mod
from typing import Any

# Module bindings retyped as `Any` to bypass Pyright stub-driven
# narrowing — every spec contract below probes documented public
# module-level helpers / instance class identifiers / mapping
# / subscript surfaces that mamba's bundled type stubs do not
# surface accurately.
os: Any = _os_mod
sys: Any = _sys_mod


_ledger: list[int] = []

# 1) os.path — `isabs` / `normpath` / `expandvars` / `relpath`
#    / `commonpath` / `commonprefix` module-level helper
#    hasattr surface
assert hasattr(os.path, "isabs") == True; _ledger.append(1)
assert hasattr(os.path, "normpath") == True; _ledger.append(1)
assert hasattr(os.path, "expandvars") == True; _ledger.append(1)
assert hasattr(os.path, "relpath") == True; _ledger.append(1)
assert hasattr(os.path, "commonpath") == True; _ledger.append(1)
assert hasattr(os.path, "commonprefix") == True; _ledger.append(1)

# 2) os — `chdir` / `putenv` mutation helper hasattr surface
assert hasattr(os, "chdir") == True; _ledger.append(1)
assert hasattr(os, "putenv") == True; _ledger.append(1)

# 3) os.environ — instance class identity + mapping protocol
assert type(os.environ).__name__ == "_Environ"; _ledger.append(1)
assert ("PATH" in os.environ) == True; _ledger.append(1)
assert os.environ.get("PATH", "MISSING") != "MISSING"; _ledger.append(1)

# 4) sys.version_info — namedtuple class identity + subscript
assert type(sys.version_info).__name__ == "version_info"; _ledger.append(1)
assert sys.version_info[0] == 3; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_os_path_environ_versioninfo_silent {sum(_ledger)} asserts")
