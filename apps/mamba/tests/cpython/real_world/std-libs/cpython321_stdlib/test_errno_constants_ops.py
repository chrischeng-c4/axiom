# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_errno_constants_ops"
# subject = "cpython321.test_errno_constants_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_errno_constants_ops.py"
# status = "filled"
# ///
"""cpython321.test_errno_constants_ops: execute CPython 3.12 seed test_errno_constants_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the `errno` module — the integer
# error-code constants and the reverse-lookup `errorcode` mapping.
# Existing fixtures only cover `signal`; this seed adds `errno`, the
# sibling module of POSIX numeric identifiers. Many `errno` constants
# diverge between Linux and Darwin (e.g. ELOOP, ENOSYS, ENOTEMPTY,
# EAGAIN), so this seed pins ONLY the cross-platform-stable subset of
# values shared by both runtimes / both kernels (POSIX values 1-34
# excluding 11, 15, 26 — the historically platform-divergent slots).
#
# Surface:
#   • errno.EPERM/ENOENT/ESRCH/EINTR/EIO/ENXIO/E2BIG/ENOEXEC/EBADF/ECHILD/
#     ENOMEM/EACCES/EFAULT/EBUSY/EEXIST/EXDEV/ENODEV/ENOTDIR/EISDIR/
#     EINVAL/ENFILE/EMFILE/ENOTTY/EFBIG/ENOSPC/ESPIPE/EROFS/EMLINK/EPIPE/
#     EDOM/ERANGE — every constant is an `int` with the canonical POSIX
#     value (1..34, excluding the platform-divergent slots);
#   • errno.errorcode — a dict mapping the integer code back to its
#     canonical mnemonic ("EPERM", etc.); used by libc strerror-style
#     formatters and by `OSError.errno` introspection helpers.
import errno
_ledger: list[int] = []

# Constant values — canonical POSIX numeric IDs (1..34 stable slots).
# Each constant must be an int and equal its canonical POSIX value.
assert errno.EPERM == 1; _ledger.append(1)
assert errno.ENOENT == 2; _ledger.append(1)
assert errno.ESRCH == 3; _ledger.append(1)
assert errno.EINTR == 4; _ledger.append(1)
assert errno.EIO == 5; _ledger.append(1)
assert errno.ENXIO == 6; _ledger.append(1)
assert errno.E2BIG == 7; _ledger.append(1)
assert errno.ENOEXEC == 8; _ledger.append(1)
assert errno.EBADF == 9; _ledger.append(1)
assert errno.ECHILD == 10; _ledger.append(1)
assert errno.ENOMEM == 12; _ledger.append(1)
assert errno.EACCES == 13; _ledger.append(1)
assert errno.EFAULT == 14; _ledger.append(1)
assert errno.EBUSY == 16; _ledger.append(1)
assert errno.EEXIST == 17; _ledger.append(1)
assert errno.EXDEV == 18; _ledger.append(1)
assert errno.ENODEV == 19; _ledger.append(1)
assert errno.ENOTDIR == 20; _ledger.append(1)
assert errno.EISDIR == 21; _ledger.append(1)
assert errno.EINVAL == 22; _ledger.append(1)
assert errno.ENFILE == 23; _ledger.append(1)
assert errno.EMFILE == 24; _ledger.append(1)
assert errno.ENOTTY == 25; _ledger.append(1)
assert errno.EFBIG == 27; _ledger.append(1)
assert errno.ENOSPC == 28; _ledger.append(1)
assert errno.ESPIPE == 29; _ledger.append(1)
assert errno.EROFS == 30; _ledger.append(1)
assert errno.EMLINK == 31; _ledger.append(1)
assert errno.EPIPE == 32; _ledger.append(1)
assert errno.EDOM == 33; _ledger.append(1)
assert errno.ERANGE == 34; _ledger.append(1)

# Type of each constant — must be int (not str, not Enum)
assert isinstance(errno.EPERM, int); _ledger.append(1)
assert isinstance(errno.ENOENT, int); _ledger.append(1)
assert isinstance(errno.EINVAL, int); _ledger.append(1)
assert isinstance(errno.ERANGE, int); _ledger.append(1)
assert isinstance(errno.EIO, int); _ledger.append(1)

# Constants must be positive (POSIX errno values are always > 0)
assert errno.EPERM > 0; _ledger.append(1)
assert errno.ENOENT > 0; _ledger.append(1)
assert errno.EINVAL > 0; _ledger.append(1)

# errorcode — reverse lookup must agree with the forward constants
assert isinstance(errno.errorcode, dict); _ledger.append(1)
assert errno.errorcode[errno.EPERM] == "EPERM"; _ledger.append(1)
assert errno.errorcode[errno.ENOENT] == "ENOENT"; _ledger.append(1)
assert errno.errorcode[errno.ESRCH] == "ESRCH"; _ledger.append(1)
assert errno.errorcode[errno.EINTR] == "EINTR"; _ledger.append(1)
assert errno.errorcode[errno.EIO] == "EIO"; _ledger.append(1)
assert errno.errorcode[errno.EBADF] == "EBADF"; _ledger.append(1)
assert errno.errorcode[errno.EACCES] == "EACCES"; _ledger.append(1)
assert errno.errorcode[errno.EEXIST] == "EEXIST"; _ledger.append(1)
assert errno.errorcode[errno.ENOTDIR] == "ENOTDIR"; _ledger.append(1)
assert errno.errorcode[errno.EISDIR] == "EISDIR"; _ledger.append(1)
assert errno.errorcode[errno.EINVAL] == "EINVAL"; _ledger.append(1)
assert errno.errorcode[errno.ENOSPC] == "ENOSPC"; _ledger.append(1)
assert errno.errorcode[errno.EROFS] == "EROFS"; _ledger.append(1)
assert errno.errorcode[errno.EPIPE] == "EPIPE"; _ledger.append(1)
assert errno.errorcode[errno.EDOM] == "EDOM"; _ledger.append(1)
assert errno.errorcode[errno.ERANGE] == "ERANGE"; _ledger.append(1)

# errorcode size — must contain at least all the stable constants
# we just probed (~31 entries minimum, real value is much higher).
assert len(errno.errorcode) >= 30; _ledger.append(1)

# errorcode values are all str
assert all(isinstance(v, str) for v in errno.errorcode.values()); _ledger.append(1)
# errorcode keys are all int
assert all(isinstance(k, int) for k in errno.errorcode.keys()); _ledger.append(1)

# Unknown error code returns None via .get
assert errno.errorcode.get(99999) is None; _ledger.append(1)
assert errno.errorcode.get(-1) is None; _ledger.append(1)

# Forward/reverse round-trip — constant → errorcode[constant] → name
# must match the attribute name we read off the module.
for name in ["EPERM", "ENOENT", "EINTR", "EBADF", "EACCES", "EEXIST",
             "EINVAL", "ENOSPC", "EROFS", "EPIPE", "EDOM", "ERANGE"]:
    code = getattr(errno, name)
    assert errno.errorcode[code] == name; _ledger.append(1)

# Distinct constants — no two stable constants share the same value.
_distinct = {errno.EPERM, errno.ENOENT, errno.ESRCH, errno.EINTR,
             errno.EIO, errno.ENXIO, errno.E2BIG, errno.ENOEXEC,
             errno.EBADF, errno.ECHILD, errno.ENOMEM, errno.EACCES,
             errno.EFAULT, errno.EBUSY, errno.EEXIST, errno.EXDEV,
             errno.ENODEV, errno.ENOTDIR, errno.EISDIR, errno.EINVAL,
             errno.ENFILE, errno.EMFILE, errno.ENOTTY, errno.EFBIG,
             errno.ENOSPC, errno.ESPIPE, errno.EROFS, errno.EMLINK,
             errno.EPIPE, errno.EDOM, errno.ERANGE}
assert len(_distinct) == 31; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_errno_constants_ops {sum(_ledger)} asserts")
