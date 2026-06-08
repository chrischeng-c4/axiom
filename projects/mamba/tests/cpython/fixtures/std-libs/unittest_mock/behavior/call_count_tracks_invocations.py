# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "behavior"
# case = "call_count_tracks_invocations"
# subject = "unittest.mock.Mock.call_count"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/testmock/testmock.py"
# status = "filled"
# ///
"""unittest.mock.Mock.call_count: call_count starts at 0 and increments by one per call; .called flips to True after the first call"""
from unittest.mock import MagicMock

m = MagicMock()
assert m.call_count == 0
assert m.called is False
m()
assert m.call_count == 1
assert m.called is True
m()
m()
assert m.call_count == 3
print("call_count_tracks_invocations OK")
