# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "behavior"
# case = "memoryview_cast_format_shape_byte_backed"
# subject = "builtins.memoryview.cast format and shape metadata"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""memoryview.cast preserves format, itemsize, shape, strides, and logical reads."""


raw = memoryview(b"\x01\x00\x02\x00")
shorts = raw.cast("H")
assert shorts.format == "H"
assert shorts.itemsize == 2
assert shorts.nbytes == 4
assert shorts.ndim == 1
assert shorts.shape == (2,)
assert shorts.strides == (2,)
assert shorts[0] == 1
assert shorts[1] == 2
assert shorts.tolist() == [1, 2]

back = shorts.cast("B")
assert back.format == "B"
assert back.itemsize == 1
assert back.shape == (4,)
assert back.tobytes() == b"\x01\x00\x02\x00"

grid = memoryview(bytearray(6)).cast("B", shape=[2, 3])
assert len(grid) == 2
assert grid.ndim == 2
assert grid.shape == (2, 3)
assert grid.strides == (3, 1)
assert grid.tolist() == [[0, 0, 0], [0, 0, 0]]
try:
    grid[0]
    assert False
except NotImplementedError:
    pass

nested = memoryview(shorts)
assert nested.format == "H"
assert nested.itemsize == 2
assert nested.shape == (2,)
assert nested[0] == 1
assert nested.tolist() == [1, 2]

print("memoryview_cast_format_shape_byte_backed: ok")
