# Operational AssertionPass seed for the value contract of `shutil`
# (file-copy + `which` + terminal-size + exception-class identity),
# `fnmatch` (glob-style string matching used by every linter / build
# tool to filter file paths), and `glob` (filesystem-level glob over a
# real tempdir). No fixture coverage yet for shutil/fnmatch/glob.
#
# The matching subset between mamba and CPython is the *operational*
# layer: `shutil.copy` reproduces the source bytes at the destination,
# `shutil.which('sh')` resolves to `/bin/sh`, `shutil.get_terminal_
# size().columns` is an int, the `SameFileError` / `Error` class names
# are exposed, every documented module-level callable is `callable`,
# `fnmatch.fnmatch` / `fnmatchcase` / `filter` honour the documented
# `*.py` semantics, `fnmatch.translate('*.py')` produces the documented
# regex string, `glob.escape('a*b') == 'a[*]b'`, `glob.glob` actually
# enumerates real files in a tempdir, and `tempfile.gettempprefix() ==
# 'tmp'`.
#
# Surface in this fixture:
#   • shutil module-level callables — copy / copytree / rmtree / move /
#     which / disk_usage / copyfile / get_terminal_size;
#   • shutil.SameFileError.__name__ == "SameFileError";
#   • shutil.Error.__name__ == "Error";
#   • shutil.get_terminal_size().columns — int;
#   • shutil.which("sh") — resolves to a non-None POSIX path
#     ending in "/sh";
#   • shutil.copy(src, dst) — destination contents match source
#     (round-trip a short byte string through a real tempdir);
#   • fnmatch.fnmatch("abc.py", "*.py") — True;
#   • fnmatch.fnmatch("abc.py", "*.txt") — False;
#   • fnmatch.fnmatchcase("AB", "*B") — True;
#   • fnmatch.filter(["a.py", "b.txt", "c.py"], "*.py") ==
#     ["a.py", "c.py"];
#   • fnmatch.translate("*.py") == "(?s:.*\\.py)\\Z";
#   • glob.escape("a*b") == "a[*]b";
#   • glob module-level callables — glob / iglob / escape;
#   • glob.glob(<tempdir>/*.py) — round-trips through a real
#     filesystem write + enumerate;
#   • tempfile.gettempprefix() == "tmp";
#   • tempfile module-level callables — TemporaryFile /
#     NamedTemporaryFile / SpooledTemporaryFile / TemporaryDirectory /
#     mkstemp / mkdtemp.
#
# Behavioral edges that DIVERGE on mamba (shutil.get_archive_formats /
# get_unpack_formats returning the documented 5-format list, shutil.
# disk_usage('/').total > 0, tempfile.gettempdir() not ending with '/',
# tempfile.TemporaryDirectory class identity + context-manager
# lifecycle, tempfile.mkdtemp creating a real directory, tempfile.
# NamedTemporaryFile usable as an instance, and fnmatch.translate
# output round-tripping through `re.match`) are covered in
# `lang_shutil_archive_tempfile_namedtemp_silent.py`.
import shutil
import fnmatch
import glob
import tempfile
import os

_ledger: list[int] = []

# 1) shutil module-level callables
assert callable(shutil.copy); _ledger.append(1)
assert callable(shutil.copytree); _ledger.append(1)
assert callable(shutil.rmtree); _ledger.append(1)
assert callable(shutil.move); _ledger.append(1)
assert callable(shutil.which); _ledger.append(1)
assert callable(shutil.disk_usage); _ledger.append(1)
assert callable(shutil.copyfile); _ledger.append(1)
assert callable(shutil.get_terminal_size); _ledger.append(1)

# 2) shutil exception-class identity
assert shutil.SameFileError.__name__ == "SameFileError"; _ledger.append(1)
assert shutil.Error.__name__ == "Error"; _ledger.append(1)

# 3) shutil.get_terminal_size().columns — int
_ts = shutil.get_terminal_size()
assert isinstance(_ts.columns, int); _ledger.append(1)
assert isinstance(_ts.lines, int); _ledger.append(1)

# 4) shutil.which — resolves POSIX "sh" to a non-None path
_sh: str | None = shutil.which("sh")
assert _sh is not None; _ledger.append(1)
assert isinstance(_sh, str); _ledger.append(1)
assert _sh.endswith("/sh"); _ledger.append(1)

# 5) shutil.copy — destination contents match source through a real
#    tempdir round-trip
_d_copy: str = tempfile.mkdtemp()
_src = os.path.join(_d_copy, "src.txt")
_dst = os.path.join(_d_copy, "dst.txt")
_fw = open(_src, "w")
_fw.write("hello")
_fw.close()
shutil.copy(_src, _dst)
_fr = open(_dst, "r")
_content: str = _fr.read()
_fr.close()
assert _content == "hello"; _ledger.append(1)
os.remove(_src)
os.remove(_dst)
os.rmdir(_d_copy)

# 6) fnmatch — glob-style string matching
assert fnmatch.fnmatch("abc.py", "*.py") == True; _ledger.append(1)
assert fnmatch.fnmatch("abc.py", "*.txt") == False; _ledger.append(1)
assert fnmatch.fnmatch("foo.bar.py", "*.py") == True; _ledger.append(1)
assert fnmatch.fnmatchcase("AB", "*B") == True; _ledger.append(1)
assert fnmatch.fnmatchcase("AB", "*b") == False; _ledger.append(1)

# 7) fnmatch.filter — applies fnmatch to every element
_pyfiles = fnmatch.filter(["a.py", "b.txt", "c.py"], "*.py")
assert _pyfiles == ["a.py", "c.py"]; _ledger.append(1)

# 8) fnmatch.translate — documented regex string
assert fnmatch.translate("*.py") == "(?s:.*\\.py)\\Z"; _ledger.append(1)

# 9) glob module callables + escape
assert callable(glob.glob); _ledger.append(1)
assert callable(glob.iglob); _ledger.append(1)
assert callable(glob.escape); _ledger.append(1)
assert glob.escape("a*b") == "a[*]b"; _ledger.append(1)
assert glob.escape("?abc") == "[?]abc"; _ledger.append(1)

# 10) glob.glob — enumerates real .py files in a tempdir
_d_glob: str = tempfile.mkdtemp()
_a = os.path.join(_d_glob, "a.py")
_b = os.path.join(_d_glob, "b.txt")
_c = os.path.join(_d_glob, "c.py")
for _p in (_a, _b, _c):
    _gf = open(_p, "w")
    _gf.write("")
    _gf.close()
_pyresults = glob.glob(os.path.join(_d_glob, "*.py"))
assert isinstance(_pyresults, list); _ledger.append(1)
assert len(_pyresults) == 2; _ledger.append(1)
for _p in (_a, _b, _c):
    os.remove(_p)
os.rmdir(_d_glob)

# 11) tempfile module-level callables + prefix
assert tempfile.gettempprefix() == "tmp"; _ledger.append(1)
assert callable(tempfile.mkstemp); _ledger.append(1)
assert callable(tempfile.mkdtemp); _ledger.append(1)
assert callable(tempfile.TemporaryFile); _ledger.append(1)
assert callable(tempfile.NamedTemporaryFile); _ledger.append(1)
assert callable(tempfile.SpooledTemporaryFile); _ledger.append(1)
assert callable(tempfile.TemporaryDirectory); _ledger.append(1)

# NB: shutil.get_archive_formats / get_unpack_formats returning the
# documented 5-format list, shutil.disk_usage('/').total > 0, tempfile.
# gettempdir() not ending with '/', tempfile.TemporaryDirectory class
# identity + context-manager lifecycle, tempfile.mkdtemp creating a
# real directory, tempfile.NamedTemporaryFile usable as an instance,
# and fnmatch.translate output round-tripping through `re.match` all
# DIVERGE on mamba — moved to the divergence-spec fixture.

print(f"MAMBA_ASSERTION_PASS: test_shutil_fnmatch_glob_value_ops {sum(_ledger)} asserts")
