# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "behavior"
# case = "patch_start_stop_manual"
# subject = "unittest.mock.patch"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/testmock/testpatch.py"
# status = "filled"
# ///
"""unittest.mock.patch: patch().start() applies the patch and the returned mock takes effect until .stop() restores the original"""
from unittest.mock import patch
import os

p = patch("os.getpid", return_value=1)
p.start()
try:
    assert os.getpid() == 1
finally:
    p.stop()
assert os.getpid() != 1
print("patch_start_stop_manual OK")
