# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os_path"
# dimension = "behavior"
# case = "expanduser_tilde_is_absolute"
# subject = "os.path.expanduser"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_posixpath.py"
# status = "filled"
# ///
"""os.path.expanduser: expanduser('~') expands the bare tilde to an absolute home directory path (only absoluteness is asserted, the value is environment-dependent)"""
import os.path

_exp = os.path.expanduser("~")
assert os.path.isabs(_exp), f"expanduser ~ is absolute = {_exp!r}"

print("expanduser_tilde_is_absolute OK")
