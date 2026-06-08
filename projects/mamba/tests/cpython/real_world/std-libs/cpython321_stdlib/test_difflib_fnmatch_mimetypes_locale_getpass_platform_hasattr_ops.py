# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_difflib_fnmatch_mimetypes_locale_getpass_platform_hasattr_ops"
# subject = "cpython321.test_difflib_fnmatch_mimetypes_locale_getpass_platform_hasattr_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_difflib_fnmatch_mimetypes_locale_getpass_platform_hasattr_ops.py"
# status = "filled"
# ///
"""cpython321.test_difflib_fnmatch_mimetypes_locale_getpass_platform_hasattr_ops: execute CPython 3.12 seed test_difflib_fnmatch_mimetypes_locale_getpass_platform_hasattr_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the value contract of the
# `difflib` / `fnmatch` / `linecache` / `mimetypes` / `locale` /
# `getpass` / `platform` seven-pack pinned to atomic 185:
# `difflib` (the documented partial module-level helper hasattr
# surface — `SequenceMatcher` / `unified_diff` /
# `get_close_matches` + the documented
# difflib.get_close_matches value contract), `fnmatch` (the
# documented full module-level helper hasattr surface —
# `fnmatch` / `fnmatchcase` / `filter` / `translate` + the
# documented fnmatch.fnmatch / fnmatchcase / filter value
# contract), `linecache` (the documented partial module-level
# helper hasattr surface — `getline` / `getlines` /
# `checkcache` / `clearcache`), `mimetypes` (the documented
# full module-level helper hasattr surface — `guess_type` /
# `guess_extension` / `guess_all_extensions` / `add_type` /
# `init` / `knownfiles` / `types_map` / `MimeTypes` + the
# documented guess_type / guess_extension value contract),
# `locale` (the documented partial module-level helper
# hasattr surface — `getlocale` / `setlocale` / `LC_ALL` /
# `LC_CTYPE` / `LC_NUMERIC` / `LC_TIME`), `getpass` (the
# documented partial module-level helper hasattr surface —
# `getuser` / `getpass` + the documented getpass.getuser() str
# return-type contract), and `platform` (the documented
# partial module-level helper hasattr surface — `system` /
# `node` / `release` / `machine` / `processor` / `platform` /
# `python_version` + the documented platform.system /
# platform.machine / platform.python_version str return-type
# contract).
#
# The matching subset between mamba and CPython is the partial
# `difflib` module hasattr surface (SequenceMatcher /
# unified_diff / get_close_matches — Differ / HtmlDiff / ndiff
# / context_diff / restore / IS_LINE_JUNK / IS_CHARACTER_JUNK
# DIVERGE + the SequenceMatcher instance .ratio value contract
# DIVERGES + the unified_diff list emission contract
# DIVERGES) + the difflib.get_close_matches value layer, the
# full `fnmatch` module hasattr surface + the full fnmatch
# value layer, the partial `linecache` module hasattr surface
# (getline / getlines / checkcache / clearcache — cache /
# lazycache DIVERGE), the full `mimetypes` module hasattr
# surface + the guess_type / guess_extension value layer, the
# partial `locale` module hasattr surface (getlocale /
# setlocale / LC_ALL / LC_CTYPE / LC_NUMERIC / LC_TIME —
# getdefaultlocale / LC_COLLATE / LC_MONETARY / LC_MESSAGES /
# Error / localeconv DIVERGE + LC_ALL integer value DIVERGES
# but is not asserted here), the partial `getpass` module
# hasattr surface (getuser / getpass — GetPassWarning
# DIVERGES) + getuser str return-type, and the partial
# `platform` module hasattr surface (system / node / release
# / machine / processor / platform / python_version —
# version / python_implementation / python_compiler /
# python_build / python_branch / architecture / uname
# DIVERGE) + the str return-type layer.
#
# Surface in this fixture:
#   • difflib — partial module hasattr surface
#     (SequenceMatcher / unified_diff / get_close_matches);
#   • difflib.get_close_matches — value contract;
#   • fnmatch — full module hasattr surface (fnmatch /
#     fnmatchcase / filter / translate);
#   • fnmatch.fnmatch / fnmatchcase / filter — value
#     contract;
#   • linecache — partial module hasattr surface (getline /
#     getlines / checkcache / clearcache);
#   • mimetypes — full module hasattr surface (guess_type /
#     guess_extension / guess_all_extensions / add_type /
#     init / knownfiles / types_map / MimeTypes);
#   • mimetypes.guess_type / guess_extension — value
#     contract;
#   • locale — partial module hasattr surface (getlocale /
#     setlocale / LC_ALL / LC_CTYPE / LC_NUMERIC / LC_TIME);
#   • getpass — partial module hasattr surface (getuser /
#     getpass);
#   • getpass.getuser — str return-type contract;
#   • platform — partial module hasattr surface (system /
#     node / release / machine / processor / platform /
#     python_version);
#   • platform.system / machine / python_version — str
#     return-type contract.
#
# Behavioral edges that DIVERGE on mamba
# (hasattr(difflib, "Differ") / "HtmlDiff" / "ndiff" /
# "context_diff" / "restore" / "IS_LINE_JUNK" /
# "IS_CHARACTER_JUNK" all False, type(SequenceMatcher(...))
# returns "float" not "SequenceMatcher", SequenceMatcher
# ratio() raises AttributeError, unified_diff list emission
# returns empty list, hasattr(linecache, "cache") /
# "lazycache" all False, hasattr(locale, "getdefaultlocale")
# / "LC_COLLATE" / "LC_MONETARY" / "LC_MESSAGES" / "Error" /
# "localeconv" all False, hasattr(getpass, "GetPassWarning")
# False, hasattr(platform, "version") /
# "python_implementation" / "python_compiler" /
# "python_build" / "python_branch" / "architecture" /
# "uname" all False) are covered in the matching spec
# fixture
# `lang_difflib_locale_platform_silent`.
import difflib
import fnmatch
import linecache
import mimetypes
import locale
import getpass
import platform


_ledger: list[int] = []

# 1) difflib — partial module hasattr surface
#    (Differ / HtmlDiff / ndiff / context_diff / restore /
#    IS_LINE_JUNK / IS_CHARACTER_JUNK DIVERGE — moved to
#    spec fixture)
assert hasattr(difflib, "SequenceMatcher") == True; _ledger.append(1)
assert hasattr(difflib, "unified_diff") == True; _ledger.append(1)
assert hasattr(difflib, "get_close_matches") == True; _ledger.append(1)

# 2) difflib.get_close_matches — value contract
_matches = difflib.get_close_matches("apple", ["ape", "apply", "apples", "apricot"])
assert _matches == ["apples", "apply", "ape"]; _ledger.append(1)

# 3) fnmatch — full module hasattr surface
assert hasattr(fnmatch, "fnmatch") == True; _ledger.append(1)
assert hasattr(fnmatch, "fnmatchcase") == True; _ledger.append(1)
assert hasattr(fnmatch, "filter") == True; _ledger.append(1)
assert hasattr(fnmatch, "translate") == True; _ledger.append(1)

# 4) fnmatch — value contract
assert fnmatch.fnmatch("a.txt", "*.txt") == True; _ledger.append(1)
assert fnmatch.fnmatch("a.txt", "*.py") == False; _ledger.append(1)
assert fnmatch.fnmatchcase("a.TXT", "*.TXT") == True; _ledger.append(1)
assert fnmatch.filter(["a.txt", "b.py", "c.txt"], "*.txt") == ["a.txt", "c.txt"]; _ledger.append(1)

# 5) linecache — partial module hasattr surface
#    (cache / lazycache DIVERGE — moved to spec fixture)
assert hasattr(linecache, "getline") == True; _ledger.append(1)
assert hasattr(linecache, "getlines") == True; _ledger.append(1)
assert hasattr(linecache, "checkcache") == True; _ledger.append(1)
assert hasattr(linecache, "clearcache") == True; _ledger.append(1)

# 6) mimetypes — full module hasattr surface
assert hasattr(mimetypes, "guess_type") == True; _ledger.append(1)
assert hasattr(mimetypes, "guess_extension") == True; _ledger.append(1)
assert hasattr(mimetypes, "guess_all_extensions") == True; _ledger.append(1)
assert hasattr(mimetypes, "add_type") == True; _ledger.append(1)
assert hasattr(mimetypes, "init") == True; _ledger.append(1)
assert hasattr(mimetypes, "knownfiles") == True; _ledger.append(1)
assert hasattr(mimetypes, "types_map") == True; _ledger.append(1)
assert hasattr(mimetypes, "MimeTypes") == True; _ledger.append(1)

# 7) mimetypes.guess_type / guess_extension — value contract
assert mimetypes.guess_type("foo.pdf") == ("application/pdf", None); _ledger.append(1)
assert mimetypes.guess_type("foo.html") == ("text/html", None); _ledger.append(1)
assert mimetypes.guess_extension("text/plain") == ".txt"; _ledger.append(1)

# 8) locale — partial module hasattr surface
#    (getdefaultlocale / LC_COLLATE / LC_MONETARY /
#    LC_MESSAGES / Error / localeconv DIVERGE — moved to
#    spec fixture)
assert hasattr(locale, "getlocale") == True; _ledger.append(1)
assert hasattr(locale, "setlocale") == True; _ledger.append(1)
assert hasattr(locale, "LC_ALL") == True; _ledger.append(1)
assert hasattr(locale, "LC_CTYPE") == True; _ledger.append(1)
assert hasattr(locale, "LC_NUMERIC") == True; _ledger.append(1)
assert hasattr(locale, "LC_TIME") == True; _ledger.append(1)

# 9) getpass — partial module hasattr surface
#    (GetPassWarning DIVERGES — moved to spec fixture)
assert hasattr(getpass, "getuser") == True; _ledger.append(1)
assert hasattr(getpass, "getpass") == True; _ledger.append(1)

# 10) getpass.getuser — str return-type contract
assert type(getpass.getuser()).__name__ == "str"; _ledger.append(1)

# 11) platform — partial module hasattr surface
#     (version / python_implementation / python_compiler /
#     python_build / python_branch / architecture / uname
#     DIVERGE — moved to spec fixture)
assert hasattr(platform, "system") == True; _ledger.append(1)
assert hasattr(platform, "node") == True; _ledger.append(1)
assert hasattr(platform, "release") == True; _ledger.append(1)
assert hasattr(platform, "machine") == True; _ledger.append(1)
assert hasattr(platform, "processor") == True; _ledger.append(1)
assert hasattr(platform, "platform") == True; _ledger.append(1)
assert hasattr(platform, "python_version") == True; _ledger.append(1)

# 12) platform.system / machine / python_version — str
#     return-type contract
assert type(platform.system()).__name__ == "str"; _ledger.append(1)
assert type(platform.machine()).__name__ == "str"; _ledger.append(1)
assert type(platform.python_version()).__name__ == "str"; _ledger.append(1)

# NB: hasattr(difflib, "Differ") / "HtmlDiff" / "ndiff" /
# "context_diff" / "restore" / "IS_LINE_JUNK" /
# "IS_CHARACTER_JUNK" all False on mamba, type
# (SequenceMatcher(...)).__name__ returns "float" on mamba,
# SequenceMatcher.ratio() raises AttributeError,
# unified_diff returns empty list, hasattr(linecache,
# "cache") / "lazycache" all False, hasattr(locale,
# "getdefaultlocale") / "LC_COLLATE" / "LC_MONETARY" /
# "LC_MESSAGES" / "Error" / "localeconv" all False,
# hasattr(getpass, "GetPassWarning") False, hasattr
# (platform, "version") / "python_implementation" /
# "python_compiler" / "python_build" / "python_branch" /
# "architecture" / "uname" all False — all DIVERGE on
# mamba — moved to the divergence-spec fixture.

print(f"MAMBA_ASSERTION_PASS: test_difflib_fnmatch_mimetypes_locale_getpass_platform_hasattr_ops {sum(_ledger)} asserts")
