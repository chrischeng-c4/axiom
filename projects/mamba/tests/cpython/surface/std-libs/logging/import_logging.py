# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "surface"
# case = "import_logging"
# subject = "logging"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""logging: import_logging (surface)."""
import logging

assert hasattr(logging, "getLogger")
print("import_logging OK")
