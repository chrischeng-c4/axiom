# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "colorsys"
# dimension = "surface"
# case = "import_colorsys"
# subject = "colorsys"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""colorsys: import_colorsys (surface)."""
import colorsys

assert hasattr(colorsys, "rgb_to_hsv")
print("import_colorsys OK")
