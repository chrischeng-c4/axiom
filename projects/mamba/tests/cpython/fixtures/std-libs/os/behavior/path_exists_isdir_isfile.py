# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "behavior"
# case = "path_exists_isdir_isfile"
# subject = "os.path.exists"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.path.exists: the cwd exists and is a dir (not a file); a nonexistent path does not exist"""
import os.path

assert os.path.exists("."), "cwd exists"
assert os.path.isdir("."), "cwd is dir"
assert not os.path.isfile("."), "cwd is not a regular file"
assert not os.path.exists("/nonexistent_path_xyz_12345"), "nonexistent path"
print("path_exists_isdir_isfile OK")
