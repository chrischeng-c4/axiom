# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "normalvariate_gauss_default_args"
# subject = "random.Random.normalvariate"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.Random.normalvariate: normalvariate() and gauss() accept default mu/sigma and return floats"""
import random

gen = random.Random(0)
assert isinstance(gen.normalvariate(), float), "normalvariate() not float"
assert isinstance(gen.gauss(), float), "gauss() not float"

print("normalvariate_gauss_default_args OK")
