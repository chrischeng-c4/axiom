# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "walrus"
# dimension = "surface"
# case = "walrus_in_comprehension_filter"
# subject = ":="
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
""":=: a walrus in a comprehension filter binds a computed value that is also used as the element expression"""
# := in a comprehension filter computes a value, reused as the element.
nums = list(range(10))
squares = [sq for n in nums if (sq := n * n) > 16]
assert squares == [25, 36, 49, 64, 81], f"squares = {squares!r}"

print("walrus_in_comprehension_filter OK")
