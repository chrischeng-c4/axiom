# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "behavior"
# case = "monotonic_non_decreasing"
# subject = "time.monotonic"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_time.py"
# status = "filled"
# ///
"""time.monotonic: ten successive time.monotonic() reads are float and never go backward (each >= the previous)"""
import time

_readings = [time.monotonic() for _ in range(10)]
for r in _readings:
    assert isinstance(r, float), f"monotonic type = {type(r)!r}"
for outer in range(len(_readings)):
    for inner in range(outer + 1, len(_readings)):
        assert _readings[inner] >= _readings[outer], \
            f"monotonic went backward at {outer},{inner}"
print("monotonic_non_decreasing OK")
