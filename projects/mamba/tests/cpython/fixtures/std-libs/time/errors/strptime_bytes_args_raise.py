# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "errors"
# case = "strptime_bytes_args_raise"
# subject = "time.strptime"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_time.py"
# status = "filled"
# ///
"""time.strptime: strptime rejects bytes for either argument: time.strptime(b'2009', '%Y') and time.strptime('2009', b'%Y') both raise TypeError"""
import time

_raised = False
try:
    time.strptime(b"2009", "%Y")
except TypeError:
    _raised = True
assert _raised, "strptime(bytes_data): expected TypeError"

_raised = False
try:
    time.strptime("2009", b"%Y")
except TypeError:
    _raised = True
assert _raised, "strptime(bytes_fmt): expected TypeError"
print("strptime_bytes_args_raise OK")
