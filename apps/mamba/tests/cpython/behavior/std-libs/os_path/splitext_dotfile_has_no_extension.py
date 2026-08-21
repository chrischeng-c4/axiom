# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os_path"
# dimension = "behavior"
# case = "splitext_dotfile_has_no_extension"
# subject = "os.path.splitext"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_posixpath.py"
# status = "filled"
# ///
"""os.path.splitext: a leading-dot hidden file has no extension; splitext('.hidden') == ('.hidden','')"""
import os.path

_n, _e = os.path.splitext(".hidden")
assert _n == ".hidden", f"dotfile name = {_n!r}"
assert _e == "", f"dotfile ext = {_e!r}"

print("splitext_dotfile_has_no_extension OK")
