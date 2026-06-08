# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest"
# dimension = "surface"
# case = "api_isolated_asyncio_test_case_is_present"
# subject = "unittest.IsolatedAsyncioTestCase"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""unittest.IsolatedAsyncioTestCase: api_isolated_asyncio_test_case_is_present (surface)."""
import unittest

assert hasattr(unittest, "IsolatedAsyncioTestCase")
print("api_isolated_asyncio_test_case_is_present OK")
