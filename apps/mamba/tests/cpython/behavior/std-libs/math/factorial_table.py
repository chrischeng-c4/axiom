# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "behavior"
# case = "factorial_table"
# subject = "math.factorial"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.factorial: math.factorial over a representative table: 0!==1, 1!==1, 5!==120, 10!==3628800, returning int"""
import math

for n, expected in [(0, 1), (1, 1), (5, 120), (10, 3628800)]:
    assert math.factorial(n) == expected, f"{n}! = {math.factorial(n)!r}"
    assert isinstance(math.factorial(n), int), f"{n}! type = {type(math.factorial(n))!r}"

print("factorial_table OK")
