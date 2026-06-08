# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "stat"
# dimension = "behavior"
# case = "permission_constants_are_octal_ints"
# subject = "stat.S_IRUSR"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_stat.py"
# status = "filled"
# ///
"""stat.S_IRUSR: permission constants are ints with the documented octal values: S_IRUSR==0o400, S_IWGRP==0o020, S_IXOTH==0o001, S_IRWXU==0o700, S_ISUID==0o4000"""
import stat

for name, value in [
    ("S_IRUSR", 0o400),
    ("S_IWGRP", 0o020),
    ("S_IXOTH", 0o001),
    ("S_IRWXU", 0o700),
    ("S_ISUID", 0o4000),
]:
    const = getattr(stat, name)
    assert isinstance(const, int), name
    assert const == value, (name, oct(const), oct(value))

print("permission_constants_are_octal_ints OK")
