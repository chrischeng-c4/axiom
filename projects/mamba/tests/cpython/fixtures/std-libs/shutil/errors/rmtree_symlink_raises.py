# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "errors"
# case = "rmtree_symlink_raises"
# subject = "shutil.rmtree"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_shutil.py"
# status = "filled"
# ///
"""shutil.rmtree: rmtree() refuses to follow a symlink-to-directory and raises OSError instead of deleting the link target (TemporaryDirectory)"""
import shutil
import tempfile
import os

_raised = False
with tempfile.TemporaryDirectory() as td:
    real = os.path.join(td, "realdir")
    link = os.path.join(td, "linkdir")
    os.mkdir(real)
    os.symlink(real, link)
    try:
        shutil.rmtree(link)
    except OSError:
        _raised = True
    # The link target must survive — rmtree refused to follow the link.
    assert os.path.isdir(real), "rmtree must not delete the symlink target"
assert _raised, "rmtree_symlink_raises: expected OSError"
print("rmtree_symlink_raises OK")
