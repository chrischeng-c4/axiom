# test_pkgutil.py — #3448 axis-1 stdlib pkgutil AssertionPass seed.
#
# Mamba-authored seed exercising the `pkgutil` module surface called out
# in the issue:
#   iter_modules over package, get_data, walk_packages, resolve_name.
#
# Surface coverage (asserts run at module scope; no helper closures per
# the mamba top-level def() quirk in test_math.py):
#   1. Module identity + public surface (hasattr).
#   2. ModuleInfo named-tuple shape.
#   3. iter_modules over a known stdlib package (json) returns
#      ModuleInfo entries with name+ispkg fields.
#   4. walk_packages over a known stdlib package — same shape.
#   5. resolve_name on already-imported modules / attributes
#      (os, os.path, builtins.int, builtins.int.from_bytes,
#       'builtins:int', 'os:path').
#   6. get_data on a known stdlib package returns bytes or None
#      (loader-dependent; assert the type is correct when present).
#
# Boxed-int dodge (subtraction-against-zero) applied for length checks.
#
# Contract with `cpython_lib_test_runner.rs`:
#   - Each `assert` runs at top level. Inverting any assert raises
#     AssertionError → non-zero exit → runner classifies as `Fail`.
#   - After every assertion has executed, the seed emits
#     `MAMBA_ASSERTION_PASS: pkgutil N asserts` to stdout.

import pkgutil
import os
import os.path
import json

_ledger: list[int] = []

# 1. Module identity + public surface.
assert pkgutil.__name__ == "pkgutil", "pkgutil.__name__"
_ledger.append(1)
assert hasattr(pkgutil, "iter_modules"), "exposes iter_modules"
_ledger.append(1)
assert hasattr(pkgutil, "walk_packages"), "exposes walk_packages"
_ledger.append(1)
assert hasattr(pkgutil, "resolve_name"), "exposes resolve_name"
_ledger.append(1)
assert hasattr(pkgutil, "get_data"), "exposes get_data"
_ledger.append(1)
assert hasattr(pkgutil, "ModuleInfo"), "exposes ModuleInfo"
_ledger.append(1)

# 2. ModuleInfo named-tuple shape.
_minfo_fields = pkgutil.ModuleInfo._fields
assert isinstance(_minfo_fields, tuple), "ModuleInfo._fields is a tuple"
_ledger.append(1)
# CPython's ModuleInfo has fields (module_finder, name, ispkg).
assert "name" in _minfo_fields, "ModuleInfo._fields includes 'name'"
_ledger.append(1)
assert "ispkg" in _minfo_fields, "ModuleInfo._fields includes 'ispkg'"
_ledger.append(1)

# 3. iter_modules over a known stdlib package (json).
_json_path = list(json.__path__)
_iter_results = list(pkgutil.iter_modules(_json_path))
assert isinstance(_iter_results, list), "iter_modules result coerces to list"
_ledger.append(1)
assert len(_iter_results) > 0, "iter_modules over json returns non-empty"
_ledger.append(1)
# Pick the first entry and exercise the ModuleInfo accessors.
_mi = _iter_results[0]
assert isinstance(_mi.name, str), "ModuleInfo.name is str"
_ledger.append(1)
assert isinstance(_mi.ispkg, bool), "ModuleInfo.ispkg is bool"
_ledger.append(1)
# Build the name set for a stable membership assertion.
_iter_names = set(m.name for m in _iter_results)
assert "decoder" in _iter_names, "iter_modules(json) lists 'decoder'"
_ledger.append(1)
assert "encoder" in _iter_names, "iter_modules(json) lists 'encoder'"
_ledger.append(1)

# 4. walk_packages over the same path. ModuleInfo shape identical.
_walk_results = list(pkgutil.walk_packages(_json_path, prefix="json."))
assert isinstance(_walk_results, list), "walk_packages result coerces to list"
_ledger.append(1)
assert len(_walk_results) > 0, "walk_packages over json returns non-empty"
_ledger.append(1)
_walk_names = set(m.name for m in _walk_results)
assert "json.decoder" in _walk_names, "walk_packages(json, prefix='json.') lists 'json.decoder'"
_ledger.append(1)
assert "json.encoder" in _walk_names, "walk_packages(json, prefix='json.') lists 'json.encoder'"
_ledger.append(1)

# 5. resolve_name on already-loaded modules + attributes.
assert pkgutil.resolve_name("os") is os, "resolve_name('os') is os"
_ledger.append(1)
assert pkgutil.resolve_name("os.path") is os.path, "resolve_name('os.path') is os.path"
_ledger.append(1)
assert pkgutil.resolve_name("os:path") is os.path, "resolve_name('os:path') is os.path"
_ledger.append(1)
assert pkgutil.resolve_name("builtins.int") is int, "resolve_name('builtins.int') is int"
_ledger.append(1)
assert pkgutil.resolve_name("builtins:int") is int, "resolve_name('builtins:int') is int"
_ledger.append(1)
assert pkgutil.resolve_name("builtins.int.from_bytes") == int.from_bytes, (
    "resolve_name('builtins.int.from_bytes') == int.from_bytes"
)
_ledger.append(1)
assert pkgutil.resolve_name("builtins:int.from_bytes") == int.from_bytes, (
    "resolve_name('builtins:int.from_bytes') == int.from_bytes"
)
_ledger.append(1)
# Path separator: os.path.pathsep is a module-level string.
assert pkgutil.resolve_name("os.path:pathsep") == os.path.pathsep, (
    "resolve_name('os.path:pathsep') == os.path.pathsep"
)
_ledger.append(1)

# Failure-case sanity: invalid names raise — assert via try/except so the
# seed sticks to the top-level pattern.
_raised = False
try:
    pkgutil.resolve_name("")
except ValueError:
    _raised = True
assert _raised == True, "resolve_name('') raises ValueError"
_ledger.append(1)
_raised = False
try:
    pkgutil.resolve_name("no_such_module_for_real_xyz")
except ImportError:
    _raised = True
assert _raised == True, "resolve_name(unknown) raises ImportError"
_ledger.append(1)

# 6. get_data — loader-dependent; assert type when present.
# json is a package; ask for its own __init__.py source.
_data = pkgutil.get_data("json", "__init__.py")
# CPython returns bytes; if the loader has no get_data, the function
# returns None. Either is contract-valid.
assert (_data is None) or isinstance(_data, bytes), (
    "get_data returns bytes or None per loader contract"
)
_ledger.append(1)

# Emit the proof-of-execution marker. Per `ASSERTION_PASS_MARKERS` in
# `cpython_lib_test_runner.rs`, presence of this token escalates the
# outcome from ImportPass to AssertionPass.
print(f"MAMBA_ASSERTION_PASS: pkgutil {len(_ledger)} asserts")
