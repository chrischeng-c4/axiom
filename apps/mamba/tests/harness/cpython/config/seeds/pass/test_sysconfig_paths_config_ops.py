# Operational AssertionPass seed for the `sysconfig` module — the
# stdlib accessor for Python's install layout (the same data the
# packaging stack reads via `pip` / `setuptools` / `installer` to
# decide where wheels land, where headers live, what `EXT_SUFFIX` is
# for native extensions). Surface focuses on the schema invariants
# (return-type + standard-key membership) rather than the concrete
# paths themselves — concrete paths differ across mamba's vendored
# Python tree and the host CPython install, so the seed compares the
# *shape* of the data: `get_path()` is `str`, `get_paths()` is a
# `dict` with the canonical 6 keys, `get_path_names()` enumerates the
# same 6 keys, `get_scheme_names()` includes `'posix_prefix'`,
# `get_python_version()` looks like `'3.x'`, `get_config_var()` of a
# missing key is `None`, `get_config_vars()` is `dict` with
# `'EXT_SUFFIX'`. No fixture coverage yet for sysconfig.
#
# Surface:
#   • sysconfig.get_path(name) → str
#       — `'purelib'`, `'platlib'`, `'stdlib'`, `'include'` each
#         return `str` (concrete value varies; just type-check);
#   • sysconfig.get_paths() → dict[str, str]
#       — keys: `purelib`, `platlib`, `stdlib`, `scripts`, `data`,
#         `include` (all six present);
#   • sysconfig.get_path_names() → iterable[str]
#       — same six canonical keys (membership-checked, not concrete
#         type — mamba returns `list`, CPython returns `tuple`);
#   • sysconfig.get_scheme_names() → iterable[str]
#       — non-empty, `'posix_prefix'` is one of them;
#   • sysconfig.get_python_version() → str (looks like 'X.Y');
#   • sysconfig.get_platform() → non-empty str;
#   • sysconfig.get_config_var('EXT_SUFFIX') → str (native-ext suffix);
#   • sysconfig.get_config_var('NONEXISTENT_KEY_XYZ') → None
#       (missing keys return `None`, NOT raise);
#   • sysconfig.get_config_vars() → dict containing `'EXT_SUFFIX'`;
#   • `hasattr(sysconfig, 'get_default_scheme')` — True on 3.10+.
import sysconfig
_ledger: list[int] = []

# get_path — every canonical name returns a str
assert isinstance(sysconfig.get_path("purelib"), str); _ledger.append(1)
assert isinstance(sysconfig.get_path("platlib"), str); _ledger.append(1)
assert isinstance(sysconfig.get_path("stdlib"), str); _ledger.append(1)
assert isinstance(sysconfig.get_path("include"), str); _ledger.append(1)
assert isinstance(sysconfig.get_path("scripts"), str); _ledger.append(1)
assert isinstance(sysconfig.get_path("data"), str); _ledger.append(1)

# get_paths — dict with all six canonical keys
_paths = sysconfig.get_paths()
assert isinstance(_paths, dict); _ledger.append(1)
assert "purelib" in _paths; _ledger.append(1)
assert "platlib" in _paths; _ledger.append(1)
assert "stdlib" in _paths; _ledger.append(1)
assert "scripts" in _paths; _ledger.append(1)
assert "data" in _paths; _ledger.append(1)
assert "include" in _paths; _ledger.append(1)
# every value is a str
assert all(isinstance(v, str) for v in _paths.values()); _ledger.append(1)
# at least six entries (some installs add more)
assert len(_paths) >= 6; _ledger.append(1)

# get_path_names — iterable enumerating the canonical key set
_names = sysconfig.get_path_names()
assert "purelib" in _names; _ledger.append(1)
assert "platlib" in _names; _ledger.append(1)
assert "stdlib" in _names; _ledger.append(1)
assert "scripts" in _names; _ledger.append(1)
assert "data" in _names; _ledger.append(1)
assert "include" in _names; _ledger.append(1)
assert len(_names) >= 6; _ledger.append(1)
# every element is a str
assert all(isinstance(n, str) for n in _names); _ledger.append(1)

# get_scheme_names — non-empty, includes 'posix_prefix' on POSIX
_schemes = sysconfig.get_scheme_names()
assert len(_schemes) > 0; _ledger.append(1)
assert "posix_prefix" in _schemes; _ledger.append(1)
assert all(isinstance(s, str) for s in _schemes); _ledger.append(1)

# get_python_version — '3.x' shape
_v = sysconfig.get_python_version()
assert isinstance(_v, str); _ledger.append(1)
assert _v.startswith("3"); _ledger.append(1)
assert "." in _v; _ledger.append(1)
# major.minor decomposes to ints
_parts = _v.split(".")
assert len(_parts) >= 2; _ledger.append(1)
assert int(_parts[0]) == 3; _ledger.append(1)
assert int(_parts[1]) >= 0; _ledger.append(1)

# get_platform — non-empty str
_plat = sysconfig.get_platform()
assert isinstance(_plat, str); _ledger.append(1)
assert len(_plat) > 0; _ledger.append(1)

# get_config_var('EXT_SUFFIX') — native-ext suffix is a str
_ext = sysconfig.get_config_var("EXT_SUFFIX")
assert isinstance(_ext, str); _ledger.append(1)

# get_config_var of a missing key returns None (NOT raises)
assert sysconfig.get_config_var("NONEXISTENT_KEY_XYZ_42") is None; _ledger.append(1)
assert sysconfig.get_config_var("__MISSING_BY_DESIGN__") is None; _ledger.append(1)
assert sysconfig.get_config_var("zzzz_not_a_real_var") is None; _ledger.append(1)

# get_config_vars — dict containing well-known build vars
_cv = sysconfig.get_config_vars()
assert isinstance(_cv, dict); _ledger.append(1)
assert "EXT_SUFFIX" in _cv; _ledger.append(1)
assert len(_cv) > 0; _ledger.append(1)

# get_default_scheme exists on Python 3.10+ (mamba also exposes it)
assert hasattr(sysconfig, "get_default_scheme"); _ledger.append(1)

# 'purelib' is in get_paths() iff it's in get_path_names()
_pn_set = set(sysconfig.get_path_names())
_p_set = set(sysconfig.get_paths().keys())
# Standard-name overlap: every name in path_names should be a key in paths
for _name in ["purelib", "platlib", "stdlib", "scripts", "data", "include"]:
    assert _name in _pn_set; _ledger.append(1)
    assert _name in _p_set; _ledger.append(1)

# Reverse — every path comes from a known name (this is the
# round-trip invariant for sysconfig's path scheme).
# Skip: some installs add custom keys beyond path_names — only check
# the canonical six are in both directions.

# get_paths() and get_path() agree for each canonical name
for _name in ["purelib", "platlib", "stdlib"]:
    assert sysconfig.get_paths()[_name] == sysconfig.get_path(_name); _ledger.append(1)

# Repeatable — calling twice returns equal results
assert sysconfig.get_python_version() == sysconfig.get_python_version(); _ledger.append(1)
assert sysconfig.get_platform() == sysconfig.get_platform(); _ledger.append(1)
assert sysconfig.get_paths() == sysconfig.get_paths(); _ledger.append(1)

# Return-type discipline — extra coverage
assert isinstance(sysconfig.get_path_names(), (list, tuple)); _ledger.append(1)
assert isinstance(sysconfig.get_scheme_names(), (list, tuple)); _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_sysconfig_paths_config_ops {sum(_ledger)} asserts")
