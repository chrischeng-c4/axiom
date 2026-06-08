# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "all_export_list_present"
# subject = "os.__all__"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.__all__: all_export_list_present (surface)."""
import os

assert hasattr(os.__all__, "__len__")
print("all_export_list_present OK")
