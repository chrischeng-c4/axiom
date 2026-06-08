# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest"
# dimension = "surface"
# case = "import_unittest"
# subject = "unittest"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""unittest: import_unittest (surface)."""
import unittest

assert hasattr(unittest, "TestCase")
print("import_unittest OK")
