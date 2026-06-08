# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "behavior"
# case = "copy_preserves_permissions"
# subject = "shutil.copy"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_shutil.py"
# status = "filled"
# ///
"""shutil.copy: copy() copies content and permission bits; a 0o644 source yields a 0o644 destination (stat.S_IMODE)"""
import shutil
import tempfile
import os
import stat

with tempfile.TemporaryDirectory() as d:
    src = os.path.join(d, "src.txt")
    dst = os.path.join(d, "dst.txt")
    with open(src, "w") as f:
        f.write("perm test")
    os.chmod(src, 0o644)
    shutil.copy(src, dst)
    dst_stat = os.stat(dst)
    assert stat.S_IMODE(dst_stat.st_mode) == 0o644, "permissions copied"

print("copy_preserves_permissions OK")
