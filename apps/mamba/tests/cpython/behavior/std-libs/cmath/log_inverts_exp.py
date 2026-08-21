# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "behavior"
# case = "log_inverts_exp"
# subject = "cmath.log"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cmath.log: log is the inverse of exp: log(exp(1+1j)) == 1+1j to tolerance"""
import cmath

_z = 1 + 1j
assert abs(cmath.log(cmath.exp(_z)) - _z) < 1e-12, "log(exp(z)) = z"
print("log_inverts_exp OK")
