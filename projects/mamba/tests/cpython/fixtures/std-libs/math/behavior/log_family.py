# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "behavior"
# case = "log_family"
# subject = "math.log"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.log: log(1)==0, log(e)==1, two-arg log(8, 2)==3, log10(1000)==3, log2(8)==3, all within 1e-10"""
import math

_eps = 1e-10
assert abs(math.log(1) - 0.0) < _eps, f"log(1) = {math.log(1)!r}"
assert abs(math.log(math.e) - 1.0) < _eps, f"log(e) = {math.log(math.e)!r}"
assert abs(math.log(8, 2) - 3.0) < _eps, f"log(8,2) = {math.log(8, 2)!r}"
assert abs(math.log10(1000) - 3.0) < _eps, f"log10(1000) = {math.log10(1000)!r}"
assert abs(math.log2(8) - 3.0) < _eps, f"log2(8) = {math.log2(8)!r}"

print("log_family OK")
