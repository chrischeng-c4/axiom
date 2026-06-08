# Operational AssertionPass seed for `posixpath` module-level
# constants and the join/basename/dirname/split surface. The
# existing `test_posixpath_ops` fixture covers `posixpath.join`,
# `basename`, `dirname`, `split`, `splitext`, `normpath`, and
# `isabs` — none of the module-level constants (`sep`, `altsep`,
# `curdir`, `pardir`, `extsep`, `pathsep`, `defpath`) are asserted
# anywhere in pass/. Surface: `posixpath.sep` is `"/"`,
# `posixpath.altsep` is None (POSIX has no alt separator),
# `posixpath.curdir` is `"."`, `posixpath.pardir` is `".."`,
# `posixpath.extsep` is `"."`, `posixpath.pathsep` is `":"`,
# `posixpath.defpath` is a non-empty `str` containing `/bin`.
import posixpath
_ledger: list[int] = []

# Separator constants
assert posixpath.sep == "/"; _ledger.append(1)
assert posixpath.altsep is None; _ledger.append(1)

# Directory token constants
assert posixpath.curdir == "."; _ledger.append(1)
assert posixpath.pardir == ".."; _ledger.append(1)

# Extension separator
assert posixpath.extsep == "."; _ledger.append(1)

# PATH-var separator
assert posixpath.pathsep == ":"; _ledger.append(1)

# Default executable search path
assert isinstance(posixpath.defpath, str); _ledger.append(1)
assert "/bin" in posixpath.defpath; _ledger.append(1)
assert len(posixpath.defpath) > 0; _ledger.append(1)

# Constant types are str (or None for altsep)
assert isinstance(posixpath.sep, str); _ledger.append(1)
assert isinstance(posixpath.curdir, str); _ledger.append(1)
assert isinstance(posixpath.pardir, str); _ledger.append(1)
assert isinstance(posixpath.extsep, str); _ledger.append(1)
assert isinstance(posixpath.pathsep, str); _ledger.append(1)

# Smoke: constants feed back into the path functions
assert posixpath.sep + "tmp" == "/tmp"; _ledger.append(1)
assert posixpath.curdir + posixpath.sep + "file" == "./file"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_posixpath_constants_ops {sum(_ledger)} asserts")
