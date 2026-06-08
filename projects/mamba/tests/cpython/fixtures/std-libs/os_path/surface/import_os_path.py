# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os_path"
# dimension = "surface"
# case = "import_os_path"
# subject = "os.path"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.path: import_os_path (surface)."""
import os

assert hasattr(os.path, "join")
print("import_os_path OK")
