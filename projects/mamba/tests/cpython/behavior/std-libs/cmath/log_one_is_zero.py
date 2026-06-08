# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "behavior"
# case = "log_one_is_zero"
# subject = "cmath.log"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cmath.log: log(1) == 0 (zero magnitude)"""
import cmath

assert abs(cmath.log(1)) < 1e-15, f"log(1) = {cmath.log(1)!r}"
print("log_one_is_zero OK")
