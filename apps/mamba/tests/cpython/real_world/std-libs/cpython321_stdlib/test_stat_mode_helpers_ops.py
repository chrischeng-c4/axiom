# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_stat_mode_helpers_ops"
# subject = "cpython321.test_stat_mode_helpers_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_stat_mode_helpers_ops.py"
# status = "filled"
# ///
"""cpython321.test_stat_mode_helpers_ops: execute CPython 3.12 seed test_stat_mode_helpers_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the `stat` module — the
# stdlib POSIX file-mode constants and the corresponding mode-
# inspection helpers (`S_ISREG`, `S_ISDIR`, `S_ISLNK`, `S_ISCHR`,
# `S_ISBLK`, `S_ISFIFO`, `S_ISSOCK`, `S_IMODE`). Used by file-type
# discrimination, permission-bit manipulation, file-mode masking,
# and any code that needs to interpret a `stat_result.st_mode`
# value. Surface focuses on the canonical permission triplets
# (`rwx` for user/group/other), the file-type bits, the setuid/
# setgid/sticky bits, and the type-test helpers — the matching
# subset between mamba and CPython. Mamba is missing `S_IFMT` —
# excluded. No fixture coverage yet for the stat module.
#
# Surface:
#   • Permission bits — user / group / other read/write/execute:
#       S_IRWXU=0o700, S_IRUSR=0o400, S_IWUSR=0o200, S_IXUSR=0o100,
#       S_IRWXG=0o070, S_IRGRP=0o040, S_IWGRP=0o020, S_IXGRP=0o010,
#       S_IRWXO=0o007, S_IROTH=0o004, S_IWOTH=0o002, S_IXOTH=0o001;
#   • File-type bits:
#       S_IFREG=0o100000 (regular), S_IFDIR=0o040000 (directory),
#       S_IFLNK=0o120000 (symlink),  S_IFCHR=0o020000 (char device),
#       S_IFBLK=0o060000 (block device), S_IFIFO=0o010000 (FIFO),
#       S_IFSOCK=0o140000 (socket);
#   • Set-id / sticky:
#       S_ISUID=0o4000, S_ISGID=0o2000, S_ISVTX=0o1000;
#   • Type-test helpers:
#       S_ISREG(m), S_ISDIR(m), S_ISLNK(m), S_ISCHR(m), S_ISBLK(m),
#       S_ISFIFO(m), S_ISSOCK(m) — return bool;
#   • Mode mask: S_IMODE(m) — returns m & 0o7777.
import stat
_ledger: list[int] = []

# User permission bits
assert stat.S_IRWXU == 0o700; _ledger.append(1)
assert stat.S_IRUSR == 0o400; _ledger.append(1)
assert stat.S_IWUSR == 0o200; _ledger.append(1)
assert stat.S_IXUSR == 0o100; _ledger.append(1)
# Group permission bits
assert stat.S_IRWXG == 0o070; _ledger.append(1)
assert stat.S_IRGRP == 0o040; _ledger.append(1)
assert stat.S_IWGRP == 0o020; _ledger.append(1)
assert stat.S_IXGRP == 0o010; _ledger.append(1)
# Other permission bits
assert stat.S_IRWXO == 0o007; _ledger.append(1)
assert stat.S_IROTH == 0o004; _ledger.append(1)
assert stat.S_IWOTH == 0o002; _ledger.append(1)
assert stat.S_IXOTH == 0o001; _ledger.append(1)

# Triplet composition — rwx is the bitwise-OR of r/w/x
assert stat.S_IRWXU == stat.S_IRUSR | stat.S_IWUSR | stat.S_IXUSR; _ledger.append(1)
assert stat.S_IRWXG == stat.S_IRGRP | stat.S_IWGRP | stat.S_IXGRP; _ledger.append(1)
assert stat.S_IRWXO == stat.S_IROTH | stat.S_IWOTH | stat.S_IXOTH; _ledger.append(1)

# File-type bits
assert stat.S_IFREG == 0o100000; _ledger.append(1)
assert stat.S_IFDIR == 0o040000; _ledger.append(1)
assert stat.S_IFLNK == 0o120000; _ledger.append(1)
assert stat.S_IFCHR == 0o020000; _ledger.append(1)
assert stat.S_IFBLK == 0o060000; _ledger.append(1)
assert stat.S_IFIFO == 0o010000; _ledger.append(1)
assert stat.S_IFSOCK == 0o140000; _ledger.append(1)

# Set-id / sticky bits
assert stat.S_ISUID == 0o4000; _ledger.append(1)
assert stat.S_ISGID == 0o2000; _ledger.append(1)
assert stat.S_ISVTX == 0o1000; _ledger.append(1)

# S_ISREG — True only on regular file mode
assert stat.S_ISREG(stat.S_IFREG) == True; _ledger.append(1)
assert stat.S_ISREG(stat.S_IFDIR) == False; _ledger.append(1)
assert stat.S_ISREG(stat.S_IFLNK) == False; _ledger.append(1)
assert stat.S_ISREG(stat.S_IFCHR) == False; _ledger.append(1)
assert stat.S_ISREG(stat.S_IFBLK) == False; _ledger.append(1)
assert stat.S_ISREG(stat.S_IFIFO) == False; _ledger.append(1)
assert stat.S_ISREG(stat.S_IFSOCK) == False; _ledger.append(1)
assert stat.S_ISREG(0o100644) == True; _ledger.append(1)

# S_ISDIR — True only on directory mode
assert stat.S_ISDIR(stat.S_IFDIR) == True; _ledger.append(1)
assert stat.S_ISDIR(stat.S_IFREG) == False; _ledger.append(1)
assert stat.S_ISDIR(stat.S_IFLNK) == False; _ledger.append(1)
assert stat.S_ISDIR(0o040755) == True; _ledger.append(1)

# S_ISLNK — True only on symlink mode
assert stat.S_ISLNK(stat.S_IFLNK) == True; _ledger.append(1)
assert stat.S_ISLNK(stat.S_IFREG) == False; _ledger.append(1)
assert stat.S_ISLNK(stat.S_IFDIR) == False; _ledger.append(1)

# S_ISCHR — True only on character-device mode
assert stat.S_ISCHR(stat.S_IFCHR) == True; _ledger.append(1)
assert stat.S_ISCHR(stat.S_IFREG) == False; _ledger.append(1)
assert stat.S_ISCHR(stat.S_IFDIR) == False; _ledger.append(1)

# S_ISBLK — True only on block-device mode
assert stat.S_ISBLK(stat.S_IFBLK) == True; _ledger.append(1)
assert stat.S_ISBLK(stat.S_IFREG) == False; _ledger.append(1)
assert stat.S_ISBLK(stat.S_IFCHR) == False; _ledger.append(1)

# S_ISFIFO — True only on FIFO mode
assert stat.S_ISFIFO(stat.S_IFIFO) == True; _ledger.append(1)
assert stat.S_ISFIFO(stat.S_IFREG) == False; _ledger.append(1)

# S_ISSOCK — True only on socket mode
assert stat.S_ISSOCK(stat.S_IFSOCK) == True; _ledger.append(1)
assert stat.S_ISSOCK(stat.S_IFREG) == False; _ledger.append(1)

# S_IMODE — masks off file-type bits, returns permission bits only
assert stat.S_IMODE(0o100644) == 0o644; _ledger.append(1)
assert stat.S_IMODE(0o040755) == 0o755; _ledger.append(1)
assert stat.S_IMODE(0o120777) == 0o777; _ledger.append(1)
assert stat.S_IMODE(0o100000) == 0o000; _ledger.append(1)
assert stat.S_IMODE(0o100777) == 0o777; _ledger.append(1)

# Type discipline
assert isinstance(stat.S_IRWXU, int); _ledger.append(1)
assert isinstance(stat.S_IFREG, int); _ledger.append(1)
assert isinstance(stat.S_ISREG(stat.S_IFREG), bool); _ledger.append(1)
assert isinstance(stat.S_ISDIR(stat.S_IFREG), bool); _ledger.append(1)
assert isinstance(stat.S_IMODE(0o100644), int); _ledger.append(1)

# Module-level attribute discipline
for _name in ["S_IRWXU", "S_IRUSR", "S_IWUSR", "S_IXUSR",
              "S_IRWXG", "S_IRGRP", "S_IWGRP", "S_IXGRP",
              "S_IRWXO", "S_IROTH", "S_IWOTH", "S_IXOTH",
              "S_IFREG", "S_IFDIR", "S_IFLNK", "S_IFCHR",
              "S_IFBLK", "S_IFIFO", "S_IFSOCK",
              "S_ISUID", "S_ISGID", "S_ISVTX",
              "S_ISREG", "S_ISDIR", "S_ISLNK", "S_ISCHR",
              "S_ISBLK", "S_ISFIFO", "S_ISSOCK", "S_IMODE"]:
    assert hasattr(stat, _name); _ledger.append(1)

# Mutual uniqueness — file-type bits are distinct
_ftypes = [stat.S_IFREG, stat.S_IFDIR, stat.S_IFLNK, stat.S_IFCHR,
           stat.S_IFBLK, stat.S_IFIFO, stat.S_IFSOCK]
assert len(_ftypes) == len(set(_ftypes)); _ledger.append(1)

# Mutual uniqueness — permission bits are distinct
_perms = [stat.S_IRUSR, stat.S_IWUSR, stat.S_IXUSR,
          stat.S_IRGRP, stat.S_IWGRP, stat.S_IXGRP,
          stat.S_IROTH, stat.S_IWOTH, stat.S_IXOTH]
assert len(_perms) == len(set(_perms)); _ledger.append(1)

# Permission bits are disjoint with file-type bits
for _p in _perms:
    for _f in _ftypes:
        assert _p & _f == 0; _ledger.append(1)

# Idempotence — same lookup, same result
assert stat.S_IRWXU == stat.S_IRWXU; _ledger.append(1)
assert stat.S_ISREG(stat.S_IFREG) == stat.S_ISREG(stat.S_IFREG); _ledger.append(1)
assert stat.S_IMODE(0o100644) == stat.S_IMODE(0o100644); _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_stat_mode_helpers_ops {sum(_ledger)} asserts")
