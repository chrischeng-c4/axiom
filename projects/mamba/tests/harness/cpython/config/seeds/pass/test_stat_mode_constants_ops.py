# Operational AssertionPass seed for the `stat` mode-test and
# permission-constant surface. Surface: the predicates
# `S_ISDIR` / `S_ISREG` / `S_ISLNK` / `S_ISCHR` / `S_ISBLK` /
# `S_ISFIFO` / `S_ISSOCK` test the file-type field of a mode int
# returned by `os.stat`; the file-type constants (S_IFDIR,
# S_IFREG, S_IFLNK, S_IFCHR, S_IFBLK, S_IFIFO, S_IFSOCK) have
# canonical octal values matching the POSIX layout; the
# permission-bit constants (S_IRUSR/W/X, S_IRGRP/W/X, S_IROTH/W/X)
# encode the rwxrwxrwx triplets, and the composite shortcuts
# S_IRWXU / S_IRWXG / S_IRWXO / S_ISUID / S_ISGID / S_ISVTX
# match their POSIX values; `S_IMODE(mode)` extracts the
# permission bits (low 12 bits including setuid/setgid/sticky).
# Companion to test_stat (which covers the broader surface).
import stat
_ledger: list[int] = []

# Mode-test predicates — positive matches
assert stat.S_ISDIR(0o040755) == True; _ledger.append(1)
assert stat.S_ISREG(0o100644) == True; _ledger.append(1)
assert stat.S_ISLNK(0o120777) == True; _ledger.append(1)
assert stat.S_ISCHR(0o020666) == True; _ledger.append(1)
assert stat.S_ISBLK(0o060666) == True; _ledger.append(1)
assert stat.S_ISFIFO(0o010644) == True; _ledger.append(1)
assert stat.S_ISSOCK(0o140644) == True; _ledger.append(1)

# Mode-test predicates — negative matches
assert stat.S_ISDIR(0o100644) == False; _ledger.append(1)
assert stat.S_ISREG(0o040755) == False; _ledger.append(1)
assert stat.S_ISLNK(0o100644) == False; _ledger.append(1)

# File-type constants — canonical POSIX octal layout
assert stat.S_IFDIR == 0o040000; _ledger.append(1)
assert stat.S_IFREG == 0o100000; _ledger.append(1)
assert stat.S_IFLNK == 0o120000; _ledger.append(1)
assert stat.S_IFCHR == 0o020000; _ledger.append(1)
assert stat.S_IFBLK == 0o060000; _ledger.append(1)
assert stat.S_IFIFO == 0o010000; _ledger.append(1)
assert stat.S_IFSOCK == 0o140000; _ledger.append(1)

# User-permission triplet (rwx)
assert stat.S_IRUSR == 0o400; _ledger.append(1)
assert stat.S_IWUSR == 0o200; _ledger.append(1)
assert stat.S_IXUSR == 0o100; _ledger.append(1)

# Group-permission triplet
assert stat.S_IRGRP == 0o040; _ledger.append(1)
assert stat.S_IWGRP == 0o020; _ledger.append(1)
assert stat.S_IXGRP == 0o010; _ledger.append(1)

# Other-permission triplet
assert stat.S_IROTH == 0o004; _ledger.append(1)
assert stat.S_IWOTH == 0o002; _ledger.append(1)
assert stat.S_IXOTH == 0o001; _ledger.append(1)

# Special bits — setuid / setgid / sticky
assert stat.S_ISUID == 0o4000; _ledger.append(1)
assert stat.S_ISGID == 0o2000; _ledger.append(1)
assert stat.S_ISVTX == 0o1000; _ledger.append(1)

# Composite rwx shortcuts
assert stat.S_IRWXU == 0o700; _ledger.append(1)
assert stat.S_IRWXG == 0o070; _ledger.append(1)
assert stat.S_IRWXO == 0o007; _ledger.append(1)

# Composite of all-rwx
assert (stat.S_IRWXU | stat.S_IRWXG | stat.S_IRWXO) == 0o777; _ledger.append(1)

# S_IMODE extracts permission bits from a full mode int
assert stat.S_IMODE(0o100644) == 0o644; _ledger.append(1)
assert stat.S_IMODE(0o040755) == 0o755; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_stat_mode_constants_ops {sum(_ledger)} asserts")
