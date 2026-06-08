# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "behavior"
# case = "heappushpop_preserves_type_on_tie"
# subject = "heapq.heappushpop"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""heapq.heappushpop: heappushpop preserves element identity/type on a numeric tie: pushing 10.0 onto [10] keeps the int 10 in the heap and returns the float 10.0"""
import heapq

# Pushing 10.0 onto [10]: 10.0 is not smaller, so 10 (the int) stays in the
# heap and the pushed float is returned.
_typed = [10]
_out = heapq.heappushpop(_typed, 10.0)
assert _typed == [10] and _out == 10.0, f"pushpop tie = {(_typed, _out)!r}"
assert type(_typed[0]) is int, f"heap kept int, got {type(_typed[0])}"
assert type(_out) is float, f"returned float, got {type(_out)}"
print("heappushpop_preserves_type_on_tie OK")
