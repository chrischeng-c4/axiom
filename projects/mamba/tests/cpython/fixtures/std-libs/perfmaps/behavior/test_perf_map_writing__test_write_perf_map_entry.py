# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "perfmaps"
# dimension = "behavior"
# case = "test_perf_map_writing__test_write_perf_map_entry"
# subject = "cpython.test_perfmaps.TestPerfMapWriting.test_write_perf_map_entry"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_perfmaps.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_perfmaps.py::TestPerfMapWriting::test_write_perf_map_entry
"""Auto-ported test: TestPerfMapWriting::test_write_perf_map_entry."""


import os
import sys


if sys.platform != "linux":
    print("TestPerfMapWriting::test_write_perf_map_entry: skipped Linux only")
else:
    from _testinternalcapi import perf_map_state_teardown, write_perf_map_entry

    assert write_perf_map_entry(0x1234, 5678, "entry1") == 0
    assert write_perf_map_entry(0x2345, 6789, "entry2") == 0
    with open(f"/tmp/perf-{os.getpid()}.map") as perf_file:
        perf_file_contents = perf_file.read()
    assert "1234 162e entry1" in perf_file_contents
    assert "2345 1a85 entry2" in perf_file_contents
    perf_map_state_teardown()
    print("TestPerfMapWriting::test_write_perf_map_entry: ok")
