# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "errors"
# case = "copyfile_same_file_symlink_raises"
# subject = "shutil.copyfile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_shutil.py"
# status = "filled"
# ///
"""shutil.copyfile: copyfile() of a file onto a symlink that points back to it raises shutil.SameFileError (src + dst symlink built in a TemporaryDirectory)"""
import shutil
import tempfile
import os

_raised = False
with tempfile.TemporaryDirectory() as td:
    src = os.path.join(td, "cheese")
    dst = os.path.join(td, "shop")
    with open(src, "w", encoding="utf-8") as f:
        f.write("cheddar")
    os.symlink("cheese", dst)  # dst -> cheese == src
    try:
        shutil.copyfile(src, dst)
    except shutil.SameFileError:
        _raised = True
assert _raised, "copyfile_same_file_symlink_raises: expected shutil.SameFileError"
print("copyfile_same_file_symlink_raises OK")
