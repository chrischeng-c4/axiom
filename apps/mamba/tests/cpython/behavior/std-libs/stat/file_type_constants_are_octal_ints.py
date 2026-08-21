# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "stat"
# dimension = "behavior"
# case = "file_type_constants_are_octal_ints"
# subject = "stat.S_IFREG"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_stat.py"
# status = "filled"
# ///
"""stat.S_IFREG: file-type constants are ints with the documented octal values: S_IFREG==0o100000, S_IFDIR==0o040000, S_IFLNK==0o120000"""
import stat

for name, value in [("S_IFREG", 0o100000), ("S_IFDIR", 0o040000), ("S_IFLNK", 0o120000)]:
    const = getattr(stat, name)
    assert isinstance(const, int), name
    assert const == value, (name, oct(const), oct(value))

print("file_type_constants_are_octal_ints OK")
