# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "getgeneratorlocals_reflects_live_frame"
# subject = "inspect.getgeneratorlocals"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.getgeneratorlocals: getgeneratorlocals() reflects the generator's live frame locals before and after stepping it once"""
import inspect

def gen(seq, a=None):
    for v in seq:
        yield v

g = gen([1, 2])
assert inspect.getgeneratorlocals(g) == {"a": None, "seq": [1, 2]}, "locals before run"
next(g)
assert inspect.getgeneratorlocals(g) == {"a": None, "seq": [1, 2], "v": 1}, "locals after step"

print("getgeneratorlocals_reflects_live_frame OK")
