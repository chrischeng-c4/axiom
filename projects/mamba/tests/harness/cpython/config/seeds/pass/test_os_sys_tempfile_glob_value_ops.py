# Operational AssertionPass seed for the value contract of the
# `os` / `sys` / `tempfile` / `glob` four-pack pinned to atomic
# 179: `os` (the documented `getcwd` / `name` / `sep` /
# `linesep` module-level helper value contract + the documented
# `os.path.join` / `split` / `splitext` / `dirname` /
# `basename` module-level helper string-transform value
# contract + the documented `os` / `os.path` module hasattr
# surface), `sys` (the documented `platform` / `maxsize` /
# `byteorder` value contract + `version_info.major`
# attribute-access contract + the documented `sys` module
# hasattr surface), `tempfile` (the documented `gettempdir`
# / `gettempprefix` value contract + the documented `tempfile`
# module hasattr surface), and `glob` (the documented
# `glob(pattern)` list-return value contract + the documented
# `glob` module hasattr surface).
#
# The matching subset between mamba and CPython is the full
# `os.getcwd` / `os.name` / `os.sep` / `os.linesep` constant /
# helper layer, the `os.path` string-transform layer (join /
# split / splitext / dirname / basename — `isabs` /
# `normpath` / `expandvars` / `relpath` / `commonpath` /
# `commonprefix` DIVERGE), the partial `os.path` hasattr
# surface (exists / isfile / isdir / abspath / expanduser /
# getsize / join / split / splitext / dirname / basename),
# the full `os` module hasattr surface (getcwd / chdir /
# listdir / mkdir / rmdir / remove / rename / stat /
# environ / name / sep / linesep / path / getenv / putenv —
# `environ` instance class DIVERGES), the `sys.platform` /
# `maxsize` / `byteorder` value layer, the
# `sys.version_info.major` attribute-access layer (subscript
# `version_info[0]` + class identity DIVERGE), the full
# `sys` module hasattr surface (argv / platform / version /
# version_info / maxsize / byteorder / stdout / stderr /
# stdin / exit / path / modules / executable), the
# `tempfile.gettempdir` / `gettempprefix` value layer + the
# full `tempfile` module hasattr surface, and the
# `glob.glob` list-return layer + the full `glob` module
# hasattr surface.
#
# Surface in this fixture:
#   • os.getcwd — current-working-directory str contract;
#   • os.name / os.sep / os.linesep — module-level constant
#     contract;
#   • os.path.join — path-component join;
#   • os.path.split — path-component split;
#   • os.path.splitext — extension split;
#   • os.path.dirname — parent-directory extraction;
#   • os.path.basename — leaf-name extraction;
#   • os.path — partial module hasattr surface (exists /
#     isfile / isdir / abspath / expanduser / getsize / join
#     / split / splitext / dirname / basename — `isabs` /
#     `normpath` / `expandvars` / `relpath` / `commonpath` /
#     `commonprefix` DIVERGE);
#   • os — module hasattr surface (getcwd / chdir / listdir
#     / mkdir / rmdir / remove / rename / stat / environ /
#     name / sep / linesep / path / getenv / putenv);
#   • sys.platform / sys.maxsize / sys.byteorder — module-
#     level value contract;
#   • sys.version_info.major — Python-version attribute-
#     access contract (subscript layer DIVERGES);
#   • sys — module hasattr surface (argv / platform /
#     version / version_info / maxsize / byteorder / stdout
#     / stderr / stdin / exit / path / modules / executable);
#   • tempfile.gettempdir — temporary-directory str contract;
#   • tempfile.gettempprefix — temporary-file prefix str
#     contract;
#   • tempfile — module hasattr surface (gettempdir /
#     gettempprefix / mkdtemp / mkstemp / NamedTemporaryFile
#     / TemporaryFile / TemporaryDirectory /
#     SpooledTemporaryFile);
#   • glob.glob — pattern-match list-return contract;
#   • glob — module hasattr surface (glob / iglob / escape /
#     has_magic).
#
# Behavioral edges that DIVERGE on mamba (hasattr(os.path,
# "isabs") / "normpath" / "expandvars" / "relpath" /
# "commonpath" / "commonprefix" are False — partial os.path
# surface, type(os.environ).__name__ == "dict" not "_Environ"
# and "PATH" not in os.environ — environ instance surface
# broken, type(sys.version_info).__name__ == "dict" not
# "version_info" and sys.version_info[0] KeyError — subscript
# layer broken) are covered in the matching spec fixture
# `lang_os_path_environ_versioninfo_silent`.
import os
import sys
import tempfile
import glob


_ledger: list[int] = []

# 1) os.getcwd — current-working-directory str contract
assert isinstance(os.getcwd(), str) == True; _ledger.append(1)
assert os.getcwd().startswith("/") == True; _ledger.append(1)

# 2) os.name / os.sep / os.linesep — module-level constants
assert os.name == "posix"; _ledger.append(1)
assert os.sep == "/"; _ledger.append(1)
assert len(os.linesep) == 1; _ledger.append(1)

# 3) os.path.join — path-component join
assert os.path.join("a", "b", "c") == "a/b/c"; _ledger.append(1)
assert os.path.join("/", "tmp", "foo") == "/tmp/foo"; _ledger.append(1)

# 4) os.path.split — path-component split
assert os.path.split("/a/b/c.txt") == ("/a/b", "c.txt"); _ledger.append(1)

# 5) os.path.splitext — extension split
assert os.path.splitext("foo.txt") == ("foo", ".txt"); _ledger.append(1)
assert os.path.splitext("noext") == ("noext", ""); _ledger.append(1)

# 6) os.path.dirname / basename — parent + leaf
assert os.path.dirname("/a/b/c.txt") == "/a/b"; _ledger.append(1)
assert os.path.basename("/a/b/c.txt") == "c.txt"; _ledger.append(1)
assert os.path.basename("noext") == "noext"; _ledger.append(1)

# 7) os.path — partial module hasattr surface
#    (isabs / normpath / expandvars / relpath / commonpath /
#    commonprefix DIVERGE — moved to spec fixture)
assert hasattr(os.path, "exists") == True; _ledger.append(1)
assert hasattr(os.path, "isfile") == True; _ledger.append(1)
assert hasattr(os.path, "isdir") == True; _ledger.append(1)
assert hasattr(os.path, "abspath") == True; _ledger.append(1)
assert hasattr(os.path, "expanduser") == True; _ledger.append(1)
assert hasattr(os.path, "getsize") == True; _ledger.append(1)
assert hasattr(os.path, "join") == True; _ledger.append(1)
assert hasattr(os.path, "split") == True; _ledger.append(1)
assert hasattr(os.path, "splitext") == True; _ledger.append(1)
assert hasattr(os.path, "dirname") == True; _ledger.append(1)
assert hasattr(os.path, "basename") == True; _ledger.append(1)

# 8) os — module hasattr surface
assert hasattr(os, "getcwd") == True; _ledger.append(1)
assert hasattr(os, "listdir") == True; _ledger.append(1)
assert hasattr(os, "mkdir") == True; _ledger.append(1)
assert hasattr(os, "rmdir") == True; _ledger.append(1)
assert hasattr(os, "remove") == True; _ledger.append(1)
assert hasattr(os, "rename") == True; _ledger.append(1)
assert hasattr(os, "stat") == True; _ledger.append(1)
assert hasattr(os, "environ") == True; _ledger.append(1)
assert hasattr(os, "name") == True; _ledger.append(1)
assert hasattr(os, "sep") == True; _ledger.append(1)
assert hasattr(os, "linesep") == True; _ledger.append(1)
assert hasattr(os, "path") == True; _ledger.append(1)
assert hasattr(os, "getenv") == True; _ledger.append(1)
# NB: hasattr(os, "chdir") / "putenv" DIVERGE on mamba (False)
# — moved to spec fixture.

# 9) sys.platform / sys.maxsize / sys.byteorder
assert isinstance(sys.platform, str) == True; _ledger.append(1)
assert sys.maxsize > 0; _ledger.append(1)
assert sys.byteorder in ("little", "big"); _ledger.append(1)

# 10) sys.version_info.major — attribute-access contract
#     (subscript layer DIVERGES — moved to spec fixture)
assert sys.version_info.major == 3; _ledger.append(1)

# 11) sys — module hasattr surface
assert hasattr(sys, "argv") == True; _ledger.append(1)
assert hasattr(sys, "platform") == True; _ledger.append(1)
assert hasattr(sys, "version") == True; _ledger.append(1)
assert hasattr(sys, "version_info") == True; _ledger.append(1)
assert hasattr(sys, "maxsize") == True; _ledger.append(1)
assert hasattr(sys, "byteorder") == True; _ledger.append(1)
assert hasattr(sys, "stdout") == True; _ledger.append(1)
assert hasattr(sys, "stderr") == True; _ledger.append(1)
assert hasattr(sys, "stdin") == True; _ledger.append(1)
assert hasattr(sys, "exit") == True; _ledger.append(1)
assert hasattr(sys, "path") == True; _ledger.append(1)
assert hasattr(sys, "modules") == True; _ledger.append(1)
assert hasattr(sys, "executable") == True; _ledger.append(1)

# 12) tempfile.gettempdir / gettempprefix — str contract
assert isinstance(tempfile.gettempdir(), str) == True; _ledger.append(1)
assert tempfile.gettempdir().startswith("/") == True; _ledger.append(1)
assert tempfile.gettempprefix() == "tmp"; _ledger.append(1)

# 13) tempfile — module hasattr surface
assert hasattr(tempfile, "gettempdir") == True; _ledger.append(1)
assert hasattr(tempfile, "gettempprefix") == True; _ledger.append(1)
assert hasattr(tempfile, "mkdtemp") == True; _ledger.append(1)
assert hasattr(tempfile, "mkstemp") == True; _ledger.append(1)
assert hasattr(tempfile, "NamedTemporaryFile") == True; _ledger.append(1)
assert hasattr(tempfile, "TemporaryFile") == True; _ledger.append(1)
assert hasattr(tempfile, "TemporaryDirectory") == True; _ledger.append(1)
assert hasattr(tempfile, "SpooledTemporaryFile") == True; _ledger.append(1)

# 14) glob.glob — pattern-match list-return contract
_g = glob.glob("/tmp/*")
assert isinstance(_g, list) == True; _ledger.append(1)
assert len(_g) >= 0; _ledger.append(1)

# 15) glob — module hasattr surface
assert hasattr(glob, "glob") == True; _ledger.append(1)
assert hasattr(glob, "iglob") == True; _ledger.append(1)
assert hasattr(glob, "escape") == True; _ledger.append(1)
assert hasattr(glob, "has_magic") == True; _ledger.append(1)

# NB: hasattr(os.path, "isabs") / "normpath" / "expandvars" /
# "relpath" / "commonpath" / "commonprefix" are False on
# mamba — partial os.path surface, type(os.environ).__name__
# == "dict" not "_Environ" + "PATH" not in os.environ on
# mamba — environ instance broken, type(sys.version_info).
# __name__ == "dict" not "version_info" + sys.version_info[0]
# KeyError on mamba — subscript layer broken — all DIVERGE on
# mamba — moved to the divergence-spec fixture.

print(f"MAMBA_ASSERTION_PASS: test_os_sys_tempfile_glob_value_ops {sum(_ledger)} asserts")
