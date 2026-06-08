# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "behavior"
# case = "byteswap_reverses_element_bytes"
# subject = "array.array"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_array.py"
# status = "filled"
# ///
"""array.array: byteswap reverses the byte order of each element; 0x0001 in an 'H' array becomes 0x0100 (256)"""
import array

bs = array.array("H", [1])  # 0x0001 -> 0x0100 = 256
bs.byteswap()
assert bs[0] == 256, f"byteswap = {bs[0]!r}"

print("byteswap_reverses_element_bytes OK")
