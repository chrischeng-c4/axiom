# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fractions"
# dimension = "surface"
# case = "import_fractions"
# subject = "fractions"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fractions: import_fractions (surface)."""
import fractions

assert hasattr(fractions, "Fraction")
print("import_fractions OK")
