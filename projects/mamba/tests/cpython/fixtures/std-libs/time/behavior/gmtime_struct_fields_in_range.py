# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "behavior"
# case = "gmtime_struct_fields_in_range"
# subject = "time.gmtime"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""time.gmtime: gmtime of an arbitrary timestamp (1_700_000_000) yields a struct_time whose mon is 1..12, mday 1..31, hour 0..23"""
import time

_st = time.gmtime(1_700_000_000)
assert 1 <= _st.tm_mon <= 12, f"month in range: {_st.tm_mon!r}"
assert 1 <= _st.tm_mday <= 31, f"mday in range: {_st.tm_mday!r}"
assert 0 <= _st.tm_hour <= 23, f"hour in range: {_st.tm_hour!r}"
print("gmtime_struct_fields_in_range OK")
