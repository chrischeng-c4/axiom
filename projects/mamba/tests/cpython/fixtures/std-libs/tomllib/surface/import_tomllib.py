# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tomllib"
# dimension = "surface"
# case = "import_tomllib"
# subject = "tomllib"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""tomllib: import_tomllib (surface)."""
import tomllib

assert hasattr(tomllib, "loads")
print("import_tomllib OK")
