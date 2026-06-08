# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "behavior"
# case = "mock_open_reads_data"
# subject = "unittest.mock.mock_open"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/testmock/testmock.py"
# status = "filled"
# ///
"""unittest.mock.mock_open: mock_open(read_data=...) provides a fake open() whose context-managed file .read() returns the configured data"""
from unittest.mock import mock_open, patch

mo = mock_open(read_data="hello world")
with patch("builtins.open", mo):
    with open("anything") as f:
        data = f.read()
assert data == "hello world"
print("mock_open_reads_data OK")
