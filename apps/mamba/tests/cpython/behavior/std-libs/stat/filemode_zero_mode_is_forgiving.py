# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "stat"
# dimension = "behavior"
# case = "filemode_zero_mode_is_forgiving"
# subject = "stat.filemode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""stat.filemode: filemode(0) is forgiving: it returns a 10-char string '?---------' rather than raising"""
import stat

# A zero mode has no recognized file-type bits and no permission bits set;
# filemode renders an unknown-type marker '?' followed by nine dashes.
result = stat.filemode(0)
assert result == "?---------", "filemode(0)"
assert len(result) == 10, "filemode(0) length"

print("filemode_zero_mode_is_forgiving OK")
