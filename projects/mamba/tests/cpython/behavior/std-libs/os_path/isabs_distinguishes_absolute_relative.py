# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os_path"
# dimension = "behavior"
# case = "isabs_distinguishes_absolute_relative"
# subject = "os.path.isabs"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_posixpath.py"
# status = "filled"
# ///
"""os.path.isabs: isabs is True only for a leading-slash path; isabs('/usr') is True, isabs('relative') and isabs('') are False"""
import os.path

assert os.path.isabs("/usr") == True, "absolute path"
assert os.path.isabs("relative") == False, "relative path"
assert os.path.isabs("") == False, "empty not absolute"

print("isabs_distinguishes_absolute_relative OK")
