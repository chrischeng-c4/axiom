# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "behavior"
# case = "perf_counter_advances"
# subject = "time.perf_counter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""time.perf_counter: time.perf_counter() is a float and strictly advances across a busy interval (two reads bracketing a 1000-iteration loop)"""
import time

_p1 = time.perf_counter()
assert isinstance(_p1, float), f"perf_counter type = {type(_p1)!r}"
for _ in range(1000):
    pass
_p2 = time.perf_counter()
assert _p2 > _p1, f"perf_counter advances: {_p1} {_p2}"
print("perf_counter_advances OK")
