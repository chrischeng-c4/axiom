# Atomic 274 pass conformance — os module (hasattr getcwd/listdir/
# mkdir/rmdir/remove/rename/environ/sep/linesep/pathsep/name/getenv/
# path/stat/walk/makedirs + getcwd is str + linesep is str + getenv
# default fallback) + stat module (hasattr S_ISDIR/S_ISREG/S_ISLNK/
# S_IFDIR/S_IFREG/S_IFLNK/S_IRUSR/S_IWUSR/S_IXUSR/filemode + S_IFDIR
# == 16384 / S_IFREG == 32768 / S_IRUSR == 256 / S_IWUSR == 128 /
# S_IXUSR == 64 + S_ISDIR(S_IFDIR) True + S_ISREG(S_IFREG) True) +
# shutil module (hasattr copy/copyfile/copytree/move/rmtree/which/
# disk_usage/make_archive/chown/get_terminal_size/copymode/copystat/
# Error/SameFileError).
# All asserts match between CPython 3.12 and mamba.
import os
import stat
import shutil


_ledger: list[int] = []

# 1) os — hasattr filesystem surface
assert hasattr(os, "getcwd") == True; _ledger.append(1)
assert hasattr(os, "listdir") == True; _ledger.append(1)
assert hasattr(os, "mkdir") == True; _ledger.append(1)
assert hasattr(os, "rmdir") == True; _ledger.append(1)
assert hasattr(os, "remove") == True; _ledger.append(1)
assert hasattr(os, "rename") == True; _ledger.append(1)
assert hasattr(os, "stat") == True; _ledger.append(1)
assert hasattr(os, "walk") == True; _ledger.append(1)
assert hasattr(os, "makedirs") == True; _ledger.append(1)

# 2) os — hasattr env/constants
assert hasattr(os, "environ") == True; _ledger.append(1)
assert hasattr(os, "sep") == True; _ledger.append(1)
assert hasattr(os, "linesep") == True; _ledger.append(1)
assert hasattr(os, "pathsep") == True; _ledger.append(1)
assert hasattr(os, "name") == True; _ledger.append(1)
assert hasattr(os, "getenv") == True; _ledger.append(1)
assert hasattr(os, "path") == True; _ledger.append(1)

# 3) os — type contracts
assert isinstance(os.getcwd(), str) == True; _ledger.append(1)
assert isinstance(os.linesep, str) == True; _ledger.append(1)
assert os.getenv("ZZZ_DEFINITELY_NOT_A_VAR_XYZ", "default") == "default"; _ledger.append(1)

# 4) stat — hasattr predicate/mode surface
assert hasattr(stat, "S_ISDIR") == True; _ledger.append(1)
assert hasattr(stat, "S_ISREG") == True; _ledger.append(1)
assert hasattr(stat, "S_ISLNK") == True; _ledger.append(1)
assert hasattr(stat, "S_IFDIR") == True; _ledger.append(1)
assert hasattr(stat, "S_IFREG") == True; _ledger.append(1)
assert hasattr(stat, "S_IFLNK") == True; _ledger.append(1)
assert hasattr(stat, "S_IRUSR") == True; _ledger.append(1)
assert hasattr(stat, "S_IWUSR") == True; _ledger.append(1)
assert hasattr(stat, "S_IXUSR") == True; _ledger.append(1)
assert hasattr(stat, "filemode") == True; _ledger.append(1)

# 5) stat — mode constant values
assert stat.S_IFDIR == 16384; _ledger.append(1)
assert stat.S_IFREG == 32768; _ledger.append(1)
assert stat.S_IRUSR == 256; _ledger.append(1)
assert stat.S_IWUSR == 128; _ledger.append(1)
assert stat.S_IXUSR == 64; _ledger.append(1)

# 6) stat — predicate value contracts
assert stat.S_ISDIR(stat.S_IFDIR) == True; _ledger.append(1)
assert stat.S_ISREG(stat.S_IFREG) == True; _ledger.append(1)

# 7) shutil — hasattr surface
assert hasattr(shutil, "copy") == True; _ledger.append(1)
assert hasattr(shutil, "copyfile") == True; _ledger.append(1)
assert hasattr(shutil, "copytree") == True; _ledger.append(1)
assert hasattr(shutil, "move") == True; _ledger.append(1)
assert hasattr(shutil, "rmtree") == True; _ledger.append(1)
assert hasattr(shutil, "which") == True; _ledger.append(1)
assert hasattr(shutil, "disk_usage") == True; _ledger.append(1)
assert hasattr(shutil, "make_archive") == True; _ledger.append(1)
assert hasattr(shutil, "chown") == True; _ledger.append(1)
assert hasattr(shutil, "get_terminal_size") == True; _ledger.append(1)
assert hasattr(shutil, "copymode") == True; _ledger.append(1)
assert hasattr(shutil, "copystat") == True; _ledger.append(1)
assert hasattr(shutil, "Error") == True; _ledger.append(1)
assert hasattr(shutil, "SameFileError") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_os_stat_shutil_value_ops {sum(_ledger)} asserts")
