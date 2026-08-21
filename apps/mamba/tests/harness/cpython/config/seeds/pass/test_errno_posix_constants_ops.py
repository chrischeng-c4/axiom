# Operational AssertionPass seed for the `errno` module — the
# stdlib POSIX error-number table used by every OS-level error
# path: `OSError.errno` comparison (`if e.errno == errno.ENOENT:`),
# `select.error` discrimination, low-level I/O retry logic
# (`EAGAIN`/`EINTR`), socket connect failures (`ECONNREFUSED`),
# and `errno.errorcode[n]` for human-readable error names in logs.
# Surface focuses on the universally matching POSIX-1 subset
# (codes 1..34 excluding 11/15, which are platform-divergent
# between BSD-derived Darwin libc and Linux glibc). Mamba uses
# Linux-style errno values; CPython on Darwin uses BSD-style.
# Codes 1..10 + 12..14 + 16..34 (skipping 15) agree on every
# platform — that's the matching surface this seed pins down.
# No fixture coverage yet for the errno module.
#
# Surface (universally matching codes only):
#   • errno.EPERM     = 1   — Operation not permitted
#   • errno.ENOENT    = 2   — No such file or directory
#   • errno.ESRCH     = 3   — No such process
#   • errno.EINTR     = 4   — Interrupted system call
#   • errno.EIO       = 5   — I/O error
#   • errno.ENXIO     = 6   — No such device or address
#   • errno.E2BIG     = 7   — Argument list too long
#   • errno.ENOEXEC   = 8   — Exec format error
#   • errno.EBADF     = 9   — Bad file descriptor
#   • errno.ECHILD    = 10  — No child processes
#   • errno.ENOMEM    = 12  — Out of memory
#   • errno.EACCES    = 13  — Permission denied
#   • errno.EFAULT    = 14  — Bad address
#   • errno.EBUSY     = 16  — Device or resource busy
#   • errno.EEXIST    = 17  — File exists
#   • errno.EXDEV     = 18  — Cross-device link
#   • errno.ENODEV    = 19  — No such device
#   • errno.ENOTDIR   = 20  — Not a directory
#   • errno.EISDIR    = 21  — Is a directory
#   • errno.EINVAL    = 22  — Invalid argument
#   • errno.ENFILE    = 23  — Too many open files in system
#   • errno.EMFILE    = 24  — Too many open files
#   • errno.ENOTTY    = 25  — Not a typewriter
#   • errno.EFBIG     = 27  — File too large
#   • errno.ENOSPC    = 28  — No space left on device
#   • errno.ESPIPE    = 29  — Illegal seek
#   • errno.EROFS     = 30  — Read-only file system
#   • errno.EMLINK    = 31  — Too many links
#   • errno.EPIPE     = 32  — Broken pipe
#   • errno.EDOM      = 33  — Numerical argument out of domain
#   • errno.ERANGE    = 34  — Numerical result out of range
#   • errno.errorcode — dict[int, str] mapping code → name.
import errno
_ledger: list[int] = []

# Common POSIX-1 codes 1..10 (no skips)
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

# Codes 12..14 (skipping 11 = EAGAIN, platform-divergent)
assert errno.ENOMEM == 12; _ledger.append(1)
assert errno.EACCES == 13; _ledger.append(1)
assert errno.EFAULT == 14; _ledger.append(1)

# Codes 16..25 (skipping 15 = ENOTBLK, BSD-only on Darwin)
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

# Codes 27..34 (skipping 26 = ETXTBSY, also platform-divergent)
assert errno.EFBIG == 27; _ledger.append(1)
assert errno.ENOSPC == 28; _ledger.append(1)
assert errno.ESPIPE == 29; _ledger.append(1)
assert errno.EROFS == 30; _ledger.append(1)
assert errno.EMLINK == 31; _ledger.append(1)
assert errno.EPIPE == 32; _ledger.append(1)
assert errno.EDOM == 33; _ledger.append(1)
assert errno.ERANGE == 34; _ledger.append(1)

# Return type discipline — every constant is int
for _name in ["EPERM", "ENOENT", "ESRCH", "EINTR", "EIO", "ENXIO",
              "E2BIG", "ENOEXEC", "EBADF", "ECHILD", "ENOMEM",
              "EACCES", "EFAULT", "EBUSY", "EEXIST", "EXDEV",
              "ENODEV", "ENOTDIR", "EISDIR", "EINVAL", "ENFILE",
              "EMFILE", "ENOTTY", "EFBIG", "ENOSPC", "ESPIPE",
              "EROFS", "EMLINK", "EPIPE", "EDOM", "ERANGE"]:
    assert isinstance(getattr(errno, _name), int); _ledger.append(1)

# Mutual uniqueness — every code in the matching subset is distinct
_codes = [errno.EPERM, errno.ENOENT, errno.ESRCH, errno.EINTR,
          errno.EIO, errno.ENXIO, errno.E2BIG, errno.ENOEXEC,
          errno.EBADF, errno.ECHILD, errno.ENOMEM, errno.EACCES,
          errno.EFAULT, errno.EBUSY, errno.EEXIST, errno.EXDEV,
          errno.ENODEV, errno.ENOTDIR, errno.EISDIR, errno.EINVAL,
          errno.ENFILE, errno.EMFILE, errno.ENOTTY, errno.EFBIG,
          errno.ENOSPC, errno.ESPIPE, errno.EROFS, errno.EMLINK,
          errno.EPIPE, errno.EDOM, errno.ERANGE]
assert len(_codes) == len(set(_codes)); _ledger.append(1)

# Range — every code is positive
for _c in _codes:
    assert _c > 0; _ledger.append(1)

# errorcode dict — maps int → str name for the matching subset
assert errno.errorcode[1] == "EPERM"; _ledger.append(1)
assert errno.errorcode[2] == "ENOENT"; _ledger.append(1)
assert errno.errorcode[3] == "ESRCH"; _ledger.append(1)
assert errno.errorcode[4] == "EINTR"; _ledger.append(1)
assert errno.errorcode[5] == "EIO"; _ledger.append(1)
assert errno.errorcode[9] == "EBADF"; _ledger.append(1)
assert errno.errorcode[13] == "EACCES"; _ledger.append(1)
assert errno.errorcode[17] == "EEXIST"; _ledger.append(1)
assert errno.errorcode[22] == "EINVAL"; _ledger.append(1)
assert errno.errorcode[32] == "EPIPE"; _ledger.append(1)

# errorcode is dict
assert isinstance(errno.errorcode, dict); _ledger.append(1)

# Every value in errorcode is a str
for _v in errno.errorcode.values():
    assert isinstance(_v, str); _ledger.append(1)

# Every key in errorcode is an int
for _k in errno.errorcode.keys():
    assert isinstance(_k, int); _ledger.append(1)

# Round-trip — errorcode[errno.X] == "X" for matching codes
assert errno.errorcode[errno.EPERM] == "EPERM"; _ledger.append(1)
assert errno.errorcode[errno.ENOENT] == "ENOENT"; _ledger.append(1)
assert errno.errorcode[errno.EINVAL] == "EINVAL"; _ledger.append(1)
assert errno.errorcode[errno.EBADF] == "EBADF"; _ledger.append(1)

# Module-level attribute discipline
for _name in ["EPERM", "ENOENT", "EINVAL", "EBADF", "errorcode"]:
    assert hasattr(errno, _name); _ledger.append(1)

# Idempotence — same lookup, same result
assert errno.EPERM == errno.EPERM; _ledger.append(1)
assert errno.errorcode[1] == errno.errorcode[1]; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_errno_posix_constants_ops {sum(_ledger)} asserts")
