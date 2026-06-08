# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest"
# dimension = "surface"
# case = "api_skip_unless_is_present"
# subject = "unittest.skipUnless"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""unittest.skipUnless: api_skip_unless_is_present (surface)."""
import unittest

assert hasattr(unittest, "skipUnless")
print("api_skip_unless_is_present OK")
