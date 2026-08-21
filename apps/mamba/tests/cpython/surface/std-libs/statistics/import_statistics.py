# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "surface"
# case = "import_statistics"
# subject = "statistics"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""statistics: import_statistics (surface)."""
import statistics

assert hasattr(statistics, "mean")
print("import_statistics OK")
