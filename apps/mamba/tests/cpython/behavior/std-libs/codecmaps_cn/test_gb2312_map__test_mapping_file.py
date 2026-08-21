# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecmaps_cn"
# dimension = "behavior"
# case = "test_gb2312_map__test_mapping_file"
# subject = "cpython.test_codecmaps_cn.TestGB2312Map.test_mapping_file"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_codecmaps_cn.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
"""GB2312 mapping table matches CPython's generated mapping-file oracle."""

import io
import unittest
from test import test_codecmaps_cn


stream = io.StringIO()
suite = unittest.TestSuite([test_codecmaps_cn.TestGB2312Map("test_mapping_file")])
result = unittest.TextTestRunner(stream=stream, verbosity=0).run(suite)

assert result.testsRun == 1, result.testsRun
assert not result.failures, stream.getvalue()
assert not result.errors, stream.getvalue()

if result.skipped:
    print("test_gb2312_map__test_mapping_file skipped:", result.skipped[0][1])
else:
    print("test_gb2312_map__test_mapping_file OK")
