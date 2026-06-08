# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest"
# dimension = "surface"
# case = "skiptest_is_attr"
# subject = "unittest"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""unittest: skiptest_is_attr (surface)."""
import unittest

assert hasattr(unittest, "SkipTest")
print("skiptest_is_attr OK")
