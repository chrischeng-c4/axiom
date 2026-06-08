# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pprint"
# dimension = "surface"
# case = "prettyprinter_is_callable"
# subject = "pprint.PrettyPrinter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""pprint.PrettyPrinter: prettyprinter_is_callable (surface)."""
import pprint

assert callable(pprint.PrettyPrinter)
print("prettyprinter_is_callable OK")
