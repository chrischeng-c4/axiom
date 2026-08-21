# Operational AssertionPass seed for SILENT divergences in `shutil`
# (get_archive_formats / get_unpack_formats returning the documented
# 5-format list, disk_usage reporting non-zero total/used/free), the
# `tempfile` lifecycle surface (gettempdir not ending with '/',
# TemporaryDirectory class identity + context-manager, mkdtemp creating
# a real directory, NamedTemporaryFile usable as an open file instance,
# TemporaryFile lifecycle), and `fnmatch.translate` whose output is
# documented to round-trip through `re.match`.
#
# The matching subset (shutil.copy file roundtrip, shutil.which,
# shutil.{SameFileError, Error} class identity, fnmatch.fnmatch /
# fnmatchcase / filter / translate value contract, glob.escape and
# glob.glob filesystem enumeration, tempfile.gettempprefix == "tmp",
# every documented callable) is covered by
# `test_shutil_fnmatch_glob_value_ops`; this fixture pins the
# CPython-only contracts that mamba currently elides.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • shutil.get_archive_formats() — list of 5 (name, description)
#     tuples (bztar / gztar / tar / xztar / zip)
#     (mamba: returns an empty list);
#   • shutil.get_unpack_formats() — list of 5 (name, extensions,
#     description) triples (mamba: returns an empty list);
#   • shutil.disk_usage("/") — (total, used, free) all positive
#     (mamba: returns (0, 640, 0));
#   • tempfile.gettempdir() — does NOT end with a trailing slash
#     (mamba: trailing-slash form);
#   • tempfile.TemporaryDirectory.__name__ == "TemporaryDirectory"
#     (mamba: returns None);
#   • tempfile.TemporaryDirectory() — context-manager that creates a
#     real directory on enter and removes it on exit
#     (mamba: AttributeError on call);
#   • tempfile.mkdtemp() — returns a real directory path that exists
#     (mamba: AttributeError on call);
#   • tempfile.NamedTemporaryFile(mode='w+', delete=False) — open
#     file with `.name` attribute (mamba: AttributeError);
#   • fnmatch.translate("*.py") — output round-trips through
#     `re.match`, matching strings ending in ".py"
#     (mamba: same string returned but mamba's `re.match` rejects
#     the inline `(?s:...)` flag group).
import shutil as _shutil_mod
import tempfile as _tempfile_mod
import fnmatch as _fnmatch_mod
import re
import os
from typing import Any

# Module bindings retyped as `Any` to bypass Pyright stub-driven
# narrowing — `shutil.get_archive_formats`, `tempfile.
# TemporaryDirectory`, and `tempfile.mkdtemp` exercise documented
# constructor surfaces that mamba's bundled type stubs do not surface.
shutil: Any = _shutil_mod
tempfile: Any = _tempfile_mod
fnmatch: Any = _fnmatch_mod

_ledger: list[int] = []

# 1) shutil.get_archive_formats() — 5 documented formats
_af: Any = shutil.get_archive_formats()
assert isinstance(_af, list); _ledger.append(1)
assert len(_af) == 5; _ledger.append(1)
_af_names = {row[0] for row in _af}
assert "zip" in _af_names; _ledger.append(1)
assert "tar" in _af_names; _ledger.append(1)
assert "gztar" in _af_names; _ledger.append(1)
assert "bztar" in _af_names; _ledger.append(1)
assert "xztar" in _af_names; _ledger.append(1)

# 2) shutil.get_unpack_formats() — 5 documented formats
_uf: Any = shutil.get_unpack_formats()
assert isinstance(_uf, list); _ledger.append(1)
assert len(_uf) == 5; _ledger.append(1)
_uf_names = {row[0] for row in _uf}
assert "zip" in _uf_names; _ledger.append(1)
assert "tar" in _uf_names; _ledger.append(1)

# 3) shutil.disk_usage — total / used / free all positive
_du: Any = shutil.disk_usage("/")
assert _du.total > 0; _ledger.append(1)
assert _du.used > 0; _ledger.append(1)
assert _du.free > 0; _ledger.append(1)

# 4) tempfile.gettempdir — does NOT end with a trailing slash
assert not tempfile.gettempdir().endswith("/"); _ledger.append(1)

# 5) tempfile.TemporaryDirectory — class identity + lifecycle
assert tempfile.TemporaryDirectory.__name__ == "TemporaryDirectory"; _ledger.append(1)
_tmp_path: str = ""
_existed_inside: bool = False
_cm: Any = tempfile.TemporaryDirectory()
_tmp_path = _cm.__enter__()
_existed_inside = os.path.isdir(_tmp_path)
_cm.__exit__(None, None, None)
_existed_after: bool = os.path.isdir(_tmp_path)
assert _existed_inside == True; _ledger.append(1)
assert _existed_after == False; _ledger.append(1)

# 6) tempfile.mkdtemp() — returns a real directory path that exists
_md: str = tempfile.mkdtemp()
assert isinstance(_md, str); _ledger.append(1)
assert os.path.isdir(_md); _ledger.append(1)
os.rmdir(_md)

# 7) tempfile.NamedTemporaryFile — open file instance with .name
_nf: Any = tempfile.NamedTemporaryFile(mode="w+", delete=False)
assert hasattr(_nf, "name"); _ledger.append(1)
assert isinstance(_nf.name, str); _ledger.append(1)
_nf.write("hello-named")
_nf.close()
_rf: Any = open(_nf.name, "r")
_nf_content: str = _rf.read()
_rf.close()
os.remove(_nf.name)
assert _nf_content == "hello-named"; _ledger.append(1)

# 8) fnmatch.translate — output round-trips through `re.match`
_pat: str = fnmatch.translate("*.py")
assert re.match(_pat, "hello.py") is not None; _ledger.append(1)
assert re.match(_pat, "hello.txt") is None; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_shutil_archive_tempfile_namedtemp_silent {sum(_ledger)} asserts")
