# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest"
# dimension = "surface"
# case = "testloader_is_type"
# subject = "unittest.TestLoader"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""unittest.TestLoader: testloader_is_type (surface)."""
import unittest

assert type(unittest.TestLoader).__name__ == "type"
print("testloader_is_type OK")
