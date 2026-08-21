# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "behavior"
# case = "copyfile_binary_roundtrip"
# subject = "shutil.copyfile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_shutil.py"
# status = "filled"
# ///
"""shutil.copyfile: copyfile copies a 256-byte binary file byte-for-byte; the destination reads back identical to the source"""
import shutil
import tempfile
import os

with tempfile.TemporaryDirectory() as d:
    src = os.path.join(d, "src.bin")
    dst = os.path.join(d, "dst.bin")
    content = bytes(range(256))
    with open(src, "wb") as f:
        f.write(content)
    shutil.copyfile(src, dst)
    with open(dst, "rb") as f2:
        assert f2.read() == content, "copyfile binary content"

print("copyfile_binary_roundtrip OK")
