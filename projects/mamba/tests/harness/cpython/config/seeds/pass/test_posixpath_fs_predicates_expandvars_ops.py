# Operational AssertionPass seed for `posixpath` filesystem-predicate
# surface — the existence / type-of-entry / mount-point probes
# (`exists`, `lexists`, `isfile`, `isdir`, `ismount`) and the
# environment-variable expansion helper `expandvars`. Existing
# `test_posixpath_ops`, `test_posixpath_constants_ops`, and
# `test_posixpath_relpath_splitdrive_ops` cover the pure-path
# manipulation API (`join`, `split`, `splitext`, `normpath`,
# `commonpath`, `relpath`, `splitdrive`, `normcase`); this seed adds
# the FS-touching predicates plus expandvars — both have stable
# behavior on Darwin and we anchor them against well-known
# always-present paths (`/`, `/etc/hosts`).
#
# Surface:
#   • posixpath.exists — True for known dirs/files, False for missing;
#   • posixpath.lexists — same as exists for non-symlink paths;
#   • posixpath.isfile — True only for regular files (e.g. /etc/hosts);
#   • posixpath.isdir — True only for directories (e.g. /, /etc);
#   • posixpath.ismount — True for '/' (root is always a mount point);
#   • posixpath.expandvars — substitutes `$VAR` / `${VAR}` from the
#     environment, leaves unknown vars untouched, and acts as identity
#     on strings with no `$` markers.
import posixpath
_ledger: list[int] = []

# exists — root always exists
assert posixpath.exists("/") == True; _ledger.append(1)
assert posixpath.exists("/etc") == True; _ledger.append(1)
assert posixpath.exists("/etc/hosts") == True; _ledger.append(1)
assert posixpath.exists("/this/path/does/not/exist/xyz") == False; _ledger.append(1)
assert posixpath.exists("") == False; _ledger.append(1)

# lexists — same as exists on non-symlink paths
assert posixpath.lexists("/") == True; _ledger.append(1)
assert posixpath.lexists("/etc") == True; _ledger.append(1)
assert posixpath.lexists("/this/path/does/not/exist/xyz") == False; _ledger.append(1)
assert posixpath.lexists("") == False; _ledger.append(1)

# isfile — only true for regular files
assert posixpath.isfile("/etc/hosts") == True; _ledger.append(1)
assert posixpath.isfile("/") == False; _ledger.append(1)
assert posixpath.isfile("/etc") == False; _ledger.append(1)
assert posixpath.isfile("/this/path/does/not/exist") == False; _ledger.append(1)
assert posixpath.isfile("") == False; _ledger.append(1)

# isdir — only true for directories
assert posixpath.isdir("/") == True; _ledger.append(1)
assert posixpath.isdir("/etc") == True; _ledger.append(1)
assert posixpath.isdir("/etc/hosts") == False; _ledger.append(1)
assert posixpath.isdir("/this/path/does/not/exist") == False; _ledger.append(1)
assert posixpath.isdir("") == False; _ledger.append(1)

# ismount — root is always a mount point on POSIX
assert posixpath.ismount("/") == True; _ledger.append(1)
assert posixpath.ismount("") == False; _ledger.append(1)
assert posixpath.ismount("/this/path/does/not/exist") == False; _ledger.append(1)

# expandvars — `$VAR` substitution from the process environment (we
# read HOME through expandvars itself rather than os.environ since
# the substitution path is what we're testing; HOME is always set on
# a Darwin / Linux user session, so the result is non-empty and is
# not the literal `$HOME` placeholder)
_home = posixpath.expandvars("$HOME")
assert _home != "$HOME"; _ledger.append(1)
assert _home != ""; _ledger.append(1)
assert posixpath.expandvars("${HOME}") == _home; _ledger.append(1)
assert posixpath.expandvars("a $HOME b").startswith("a "); _ledger.append(1)
assert posixpath.expandvars("a $HOME b").endswith(" b"); _ledger.append(1)
assert _home in posixpath.expandvars("$HOME/file.txt"); _ledger.append(1)

# expandvars — unknown vars are left as-is
assert posixpath.expandvars("$THIS_VAR_DOES_NOT_EXIST_XYZ") == "$THIS_VAR_DOES_NOT_EXIST_XYZ"; _ledger.append(1)
assert posixpath.expandvars("${THIS_VAR_DOES_NOT_EXIST_XYZ}") == "${THIS_VAR_DOES_NOT_EXIST_XYZ}"; _ledger.append(1)

# expandvars — identity on no-`$` strings
assert posixpath.expandvars("plain/path/no/vars") == "plain/path/no/vars"; _ledger.append(1)
assert posixpath.expandvars("no_vars_here") == "no_vars_here"; _ledger.append(1)
assert posixpath.expandvars("") == ""; _ledger.append(1)

# expandvars — naked `$` is left as-is
assert posixpath.expandvars("$") == "$"; _ledger.append(1)
assert posixpath.expandvars("just $ alone") == "just $ alone"; _ledger.append(1)

# Composition — exists ↔ isfile-or-isdir (mutually exclusive on real paths)
assert posixpath.exists("/etc/hosts") == (posixpath.isfile("/etc/hosts") or posixpath.isdir("/etc/hosts")); _ledger.append(1)
assert posixpath.exists("/") == (posixpath.isfile("/") or posixpath.isdir("/")); _ledger.append(1)
assert posixpath.isfile("/etc/hosts") and not posixpath.isdir("/etc/hosts"); _ledger.append(1)
assert posixpath.isdir("/etc") and not posixpath.isfile("/etc"); _ledger.append(1)

# isfile-on-missing implies not-exists
assert not posixpath.isfile("/nonexistent/xyz"); _ledger.append(1)
assert not posixpath.exists("/nonexistent/xyz"); _ledger.append(1)

# Boolean type — predicates return bool, not 0/1 or None
assert isinstance(posixpath.exists("/"), bool); _ledger.append(1)
assert isinstance(posixpath.isfile("/etc/hosts"), bool); _ledger.append(1)
assert isinstance(posixpath.isdir("/"), bool); _ledger.append(1)
assert isinstance(posixpath.ismount("/"), bool); _ledger.append(1)
assert isinstance(posixpath.lexists("/"), bool); _ledger.append(1)

# expandvars return type
assert isinstance(posixpath.expandvars("$HOME"), str); _ledger.append(1)
assert isinstance(posixpath.expandvars(""), str); _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_posixpath_fs_predicates_expandvars_ops {sum(_ledger)} asserts")
