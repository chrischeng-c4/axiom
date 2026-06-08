# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os_path"
# dimension = "behavior"
# case = "split_head_tail"
# subject = "os.path.split"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_posixpath.py"
# status = "filled"
# ///
"""os.path.split: split returns (head, tail); '/usr/local/bin/python' splits to ('/usr/local/bin','python') and a bare 'file.py' to ('','file.py')"""
import os.path

_h, _t = os.path.split("/usr/local/bin/python")
assert _h == "/usr/local/bin", f"split head = {_h!r}"
assert _t == "python", f"split tail = {_t!r}"

_h2, _t2 = os.path.split("file.py")
assert _h2 == "", f"no dir head = {_h2!r}"
assert _t2 == "file.py", f"no dir tail = {_t2!r}"

print("split_head_tail OK")
