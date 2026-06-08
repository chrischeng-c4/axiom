# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest"
# dimension = "surface"
# case = "api_install_handler_is_present"
# subject = "unittest.installHandler"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""unittest.installHandler: api_install_handler_is_present (surface)."""
import unittest

assert hasattr(unittest, "installHandler")
print("api_install_handler_is_present OK")
