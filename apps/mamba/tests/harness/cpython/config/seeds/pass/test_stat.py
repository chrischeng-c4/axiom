import stat

_ledger: list[int] = []

# File-type predicates on raw st_mode integers
assert stat.S_ISDIR(0o40755), "S_ISDIR recognizes 0o40755 as a directory"
_ledger.append(1)

assert not stat.S_ISDIR(0o100644), "S_ISDIR rejects a regular file mode"
_ledger.append(1)

assert stat.S_ISREG(0o100644), "S_ISREG recognizes a regular file"
_ledger.append(1)

assert not stat.S_ISREG(0o40755), "S_ISREG rejects a directory mode"
_ledger.append(1)

assert stat.S_ISLNK(0o120777), "S_ISLNK recognizes a symbolic link"
_ledger.append(1)

assert stat.S_ISFIFO(0o010644), "S_ISFIFO recognizes a fifo"
_ledger.append(1)

assert stat.S_ISCHR(0o020644), "S_ISCHR recognizes a character device"
_ledger.append(1)

assert stat.S_ISBLK(0o060644), "S_ISBLK recognizes a block device"
_ledger.append(1)

# Permission-bit constants follow the POSIX octal layout
assert stat.S_IRUSR == 0o400, "S_IRUSR is owner read bit"
_ledger.append(1)

assert stat.S_IWUSR == 0o200, "S_IWUSR is owner write bit"
_ledger.append(1)

assert stat.S_IXUSR == 0o100, "S_IXUSR is owner execute bit"
_ledger.append(1)

assert stat.S_IRWXU == 0o700, "S_IRWXU is full owner permission"
_ledger.append(1)

# S_IMODE strips the file-type bits
assert stat.S_IMODE(0o100755) == 0o755, "S_IMODE keeps permission bits only"
_ledger.append(1)

# filemode renders a Unix `ls -l`-style string
assert stat.filemode(0o100644) == "-rw-r--r--", "filemode renders rw-r--r--"
_ledger.append(1)

# ST_* index constants for os.stat() result tuples
assert stat.ST_MODE == 0, "ST_MODE is index 0"
_ledger.append(1)

assert stat.ST_SIZE == 6, "ST_SIZE is index 6"
_ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_stat {sum(_ledger)} asserts")
