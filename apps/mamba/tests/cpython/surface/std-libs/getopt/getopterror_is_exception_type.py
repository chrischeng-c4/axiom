# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "getopt"
# dimension = "surface"
# case = "getopterror_is_exception_type"
# subject = "getopt.GetoptError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""getopt.GetoptError: getopterror_is_exception_type (surface)."""
import getopt

assert hasattr(getopt.GetoptError, "__cause__")
print("getopterror_is_exception_type OK")
