# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "mixed_numeric_inference"
# dimension = "behavior"
# case = "float_floor_division_is_float"
# subject = "float // int floor division returns a whole-valued float"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""float // int floors but stays float (7.0 // 2 == 3.0 with float type, not int 3)."""
fd = 7.0 // 2
assert fd == 3.0, fd
assert isinstance(fd, float), type(fd)
assert (7 // 2.0) == 3.0, 7 // 2.0
assert isinstance(7 // 2.0, float), type(7 // 2.0)
print("float_floor_division_is_float OK")
