# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "behavior"
# case = "expm1_log1p_small_arg"
# subject = "math.expm1"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.expm1: small-arg precision builtins: expm1(0.0)==0.0, expm1(1.0)==1.7182818284590453, log1p(0.0)==0.0, log1p(1.0)==0.6931471805599453"""
import math

assert math.expm1(0.0) == 0.0, f"expm1(0.0) = {math.expm1(0.0)!r}"
assert math.expm1(1.0) == 1.7182818284590453, f"expm1(1.0) = {math.expm1(1.0)!r}"
assert math.log1p(0.0) == 0.0, f"log1p(0.0) = {math.log1p(0.0)!r}"
assert math.log1p(1.0) == 0.6931471805599453, f"log1p(1.0) = {math.log1p(1.0)!r}"

print("expm1_log1p_small_arg OK")
