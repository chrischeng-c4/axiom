# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "errors"
# case = "sleep_non_number_raises"
# subject = "time.sleep"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_time.py"
# status = "filled"
# ///
"""time.sleep: sleep requires a real number; passing a list, a str, or a complex each raises TypeError"""
import time

for _bad in ([], "a", complex(0, 0)):
    _raised = False
    try:
        time.sleep(_bad)
    except TypeError:
        _raised = True
    assert _raised, f"sleep({type(_bad).__name__}): expected TypeError"
print("sleep_non_number_raises OK")
