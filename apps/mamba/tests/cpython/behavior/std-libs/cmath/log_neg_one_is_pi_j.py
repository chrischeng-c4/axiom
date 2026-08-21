# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "behavior"
# case = "log_neg_one_is_pi_j"
# subject = "cmath.log"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cmath.log: log(-1) == pi*j on the principal branch"""
import cmath
import math

assert abs(cmath.log(-1) - 1j * math.pi) < 1e-15, f"log(-1) = {cmath.log(-1)!r}"
print("log_neg_one_is_pi_j OK")
