# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "surface"
# case = "import_difflib"
# subject = "difflib"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""difflib: import_difflib (surface)."""
import difflib

assert hasattr(difflib, "SequenceMatcher")
print("import_difflib OK")
