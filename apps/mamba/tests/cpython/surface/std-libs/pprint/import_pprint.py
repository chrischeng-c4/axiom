# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pprint"
# dimension = "surface"
# case = "import_pprint"
# subject = "pprint"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""pprint: import_pprint (surface)."""
import pprint

assert hasattr(pprint, "pformat")
print("import_pprint OK")
