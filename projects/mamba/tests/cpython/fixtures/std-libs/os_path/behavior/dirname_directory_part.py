# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os_path"
# dimension = "behavior"
# case = "dirname_directory_part"
# subject = "os.path.dirname"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_posixpath.py"
# status = "filled"
# ///
"""os.path.dirname: dirname returns the directory part; '/usr/local/bin/python' -> '/usr/local/bin' and a bare 'file.py' -> '' """
import os.path

assert os.path.dirname("/usr/local/bin/python") == "/usr/local/bin", "dirname"
assert os.path.dirname("file.py") == "", "dirname no dir"

print("dirname_directory_part OK")
