# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "stat"
# dimension = "behavior"
# case = "s_ifmt_extracts_file_type_bits"
# subject = "stat.S_IFMT"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_stat.py"
# status = "filled"
# ///
"""stat.S_IFMT: S_IFMT extracts only the file-type bits: S_IFMT(0o100644) == S_IFREG == 0o100000 and S_IFMT(0o040755) == S_IFDIR"""
import stat

# S_IFMT keeps the file-type bits and drops the permission bits.
assert stat.S_IFMT(0o100644) == stat.S_IFREG, "S_IFMT(reg) == S_IFREG"
assert stat.S_IFMT(0o100644) == 0o100000, "S_IFMT(reg) octal"
assert stat.S_IFMT(0o040755) == stat.S_IFDIR, "S_IFMT(dir) == S_IFDIR"
assert stat.S_IFMT(0o120755) == stat.S_IFLNK, "S_IFMT(lnk) == S_IFLNK"

print("s_ifmt_extracts_file_type_bits OK")
