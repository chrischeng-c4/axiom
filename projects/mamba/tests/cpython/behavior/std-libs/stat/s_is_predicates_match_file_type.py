# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "stat"
# dimension = "behavior"
# case = "s_is_predicates_match_file_type"
# subject = "stat.S_ISDIR"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_stat.py"
# status = "filled"
# ///
"""stat.S_ISDIR: each S_IS* predicate returns True only for its own file-type mode and False for a regular-file mode: S_ISDIR(0o040755), S_ISREG(0o100644), S_ISLNK(0o120755), S_ISFIFO(0o010000), S_ISCHR(0o020000), S_ISBLK(0o060000), S_ISSOCK(0o140000)"""
import stat

# Each predicate is True for its own file-type mode.
assert stat.S_ISDIR(0o040755) is True, "S_ISDIR(dir)"
assert stat.S_ISREG(0o100644) is True, "S_ISREG(reg)"
assert stat.S_ISLNK(0o120755) is True, "S_ISLNK(lnk)"
assert stat.S_ISFIFO(0o010000) is True, "S_ISFIFO(fifo)"
assert stat.S_ISCHR(0o020000) is True, "S_ISCHR(chr)"
assert stat.S_ISBLK(0o060000) is True, "S_ISBLK(blk)"
assert stat.S_ISSOCK(0o140000) is True, "S_ISSOCK(sock)"

# A regular-file mode is rejected by the non-regular predicates.
reg = 0o100644
assert stat.S_ISDIR(reg) is False, "S_ISDIR(reg)"
assert stat.S_ISLNK(reg) is False, "S_ISLNK(reg)"
assert stat.S_ISFIFO(reg) is False, "S_ISFIFO(reg)"
assert stat.S_ISCHR(reg) is False, "S_ISCHR(reg)"
assert stat.S_ISBLK(reg) is False, "S_ISBLK(reg)"
assert stat.S_ISSOCK(reg) is False, "S_ISSOCK(reg)"

print("s_is_predicates_match_file_type OK")
