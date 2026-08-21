# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os_path"
# dimension = "behavior"
# case = "splitext_name_and_extension"
# subject = "os.path.splitext"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_posixpath.py"
# status = "filled"
# ///
"""os.path.splitext: splitext peels the last extension; 'file.py' -> ('file','.py'), 'archive.tar.gz' -> ('archive.tar','.gz'), 'noext' -> ('noext','')"""
import os.path

_name, _ext = os.path.splitext("file.py")
assert _name == "file", f"splitext name = {_name!r}"
assert _ext == ".py", f"splitext ext = {_ext!r}"

_name2, _ext2 = os.path.splitext("archive.tar.gz")
assert _name2 == "archive.tar", f"splitext multiple dots = {_name2!r}"
assert _ext2 == ".gz", f"splitext last ext = {_ext2!r}"

_name3, _ext3 = os.path.splitext("noext")
assert _name3 == "noext", f"no ext name = {_name3!r}"
assert _ext3 == "", f"no ext = {_ext3!r}"

print("splitext_name_and_extension OK")
