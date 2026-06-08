# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "stat"
# dimension = "behavior"
# case = "s_imode_strips_file_type_bits"
# subject = "stat.S_IMODE"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""stat.S_IMODE: S_IMODE strips the file-type bits and keeps only the permission bits: S_IMODE(0o100755) == 0o755"""
import stat

# A regular file (0o100000) with rwxr-xr-x perms keeps only the 0o755 perms.
assert stat.S_IMODE(0o100755) == 0o755, "S_IMODE(0o100755)"
assert oct(stat.S_IMODE(0o100755)) == "0o755", "oct(S_IMODE(0o100755))"
# A directory (0o040000) with 700 perms keeps only 0o700.
assert stat.S_IMODE(0o040700) == 0o700, "S_IMODE(0o040700)"

print("s_imode_strips_file_type_bits OK")
