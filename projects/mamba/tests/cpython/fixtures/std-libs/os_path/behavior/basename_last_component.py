# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os_path"
# dimension = "behavior"
# case = "basename_last_component"
# subject = "os.path.basename"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_posixpath.py"
# status = "filled"
# ///
"""os.path.basename: basename returns the final component; '/usr/local/bin/python' -> 'python', 'file.py' -> 'file.py', and a trailing slash -> '' """
import os.path

assert os.path.basename("/usr/local/bin/python") == "python", "basename"
assert os.path.basename("file.py") == "file.py", "basename no dir"
assert os.path.basename("/usr/local/") == "", "basename trailing slash"

print("basename_last_component OK")
