# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shlex"
# dimension = "surface"
# case = "import_shlex"
# subject = "shlex"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""shlex: import_shlex (surface)."""
import shlex

assert hasattr(shlex, "split")
print("import_shlex OK")
