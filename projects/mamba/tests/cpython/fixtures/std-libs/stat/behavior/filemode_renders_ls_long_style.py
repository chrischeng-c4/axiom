# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "stat"
# dimension = "behavior"
# case = "filemode_renders_ls_long_style"
# subject = "stat.filemode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_stat.py"
# status = "filled"
# ///
"""stat.filemode: filemode renders an ls -l style string: filemode(0o100644) == '-rw-r--r--' and filemode(0o040755) == 'drwxr-xr-x'"""
import stat

# Regular file, 644 perms -> leading '-' and rw-r--r-- triplets.
assert stat.filemode(0o100644) == "-rw-r--r--", "filemode(reg 644)"
# Directory, 755 perms -> leading 'd' and rwxr-xr-x triplets.
assert stat.filemode(0o040755) == "drwxr-xr-x", "filemode(dir 755)"

print("filemode_renders_ls_long_style OK")
