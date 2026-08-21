# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "behavior"
# case = "dist_euclidean"
# subject = "math.dist"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.dist: math.dist computes Euclidean distance over equal-length point sequences: dist([0,0],[3,4])==5.0, dist([1,2,3],[4,6,15])==13.0, and accepts tuple inputs"""
import math

assert math.dist([0, 0], [3, 4]) == 5.0, f"dist([0,0],[3,4]) = {math.dist([0, 0], [3, 4])!r}"
assert math.dist([1, 2, 3], [4, 6, 15]) == 13.0, f"dist 3d = {math.dist([1, 2, 3], [4, 6, 15])!r}"
assert math.dist([0], [0]) == 0.0, f"dist([0],[0]) = {math.dist([0], [0])!r}"
assert math.dist((1.5, 2.5), (1.5, 2.5)) == 0.0, "tuple inputs accepted"

print("dist_euclidean OK")
