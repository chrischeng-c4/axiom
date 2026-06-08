# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "behavior"
# case = "seek_constants_values"
# subject = "io.SEEK_SET"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""io.SEEK_SET: the seek-origin constants are SEEK_SET == 0, SEEK_CUR == 1, SEEK_END == 2"""
import io

assert io.SEEK_SET == 0, f"SEEK_SET = {io.SEEK_SET!r}"
assert io.SEEK_CUR == 1, f"SEEK_CUR = {io.SEEK_CUR!r}"
assert io.SEEK_END == 2, f"SEEK_END = {io.SEEK_END!r}"

print("seek_constants_values OK")
