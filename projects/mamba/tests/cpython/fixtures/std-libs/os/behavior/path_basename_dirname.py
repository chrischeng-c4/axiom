# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "behavior"
# case = "path_basename_dirname"
# subject = "os.path.basename"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.path.basename: basename/dirname of '/tmp/dir/file.txt' are 'file.txt' and '/tmp/dir'"""
import os.path

path = "/tmp/dir/file.txt"
assert os.path.basename(path) == "file.txt", f"basename = {os.path.basename(path)!r}"
assert os.path.dirname(path) == "/tmp/dir", f"dirname = {os.path.dirname(path)!r}"
print("path_basename_dirname OK")
