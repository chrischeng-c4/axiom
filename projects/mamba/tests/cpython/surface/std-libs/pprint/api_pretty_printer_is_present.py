# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pprint"
# dimension = "surface"
# case = "api_pretty_printer_is_present"
# subject = "pprint.PrettyPrinter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pprint.PrettyPrinter: api_pretty_printer_is_present (surface)."""
import pprint

assert hasattr(pprint, "PrettyPrinter")
print("api_pretty_printer_is_present OK")
