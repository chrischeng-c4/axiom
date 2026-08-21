# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "errors"
# case = "insane_timestamp_overflows"
# subject = "time.gmtime"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_time.py"
# status = "filled"
# ///
"""time.gmtime: an out-of-range timestamp (1e200) raises OverflowError (not a garbage struct_time) across ctime, gmtime, and localtime"""
import time

for _fn in (time.ctime, time.gmtime, time.localtime):
    _raised = False
    try:
        _fn(1e200)
    except OverflowError:
        _raised = True
    assert _raised, f"{_fn.__name__}(1e200): expected OverflowError"
print("insane_timestamp_overflows OK")
