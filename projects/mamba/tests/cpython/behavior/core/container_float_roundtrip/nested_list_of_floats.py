# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "container_float_roundtrip"
# dimension = "behavior"
# case = "nested_list_of_floats"
# subject = "float read back through a nested list-of-list"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A float in a list-of-lists must read back as the same float at [i][j]."""
grid = [[1.5, 2.5], [3.5, 4.5]]
assert grid[0][0] == 1.5, grid[0][0]
assert grid[0][1] == 2.5, grid[0][1]
assert grid[1][0] == 3.5, grid[1][0]
assert grid[1][1] == 4.5, grid[1][1]
assert isinstance(grid[1][1], float), type(grid[1][1])
print("nested_list_of_floats OK")
