# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "behavior"
# case = "double_tobytes_frombytes_roundtrip"
# subject = "array.array"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_array.py"
# status = "filled"
# ///
"""array.array: a 'd' array round-trips through tobytes()/frombytes(); 3 doubles serialize to 24 bytes and reload to the same values"""
import array

a = array.array("d", [1.1, 2.2, 3.3])
raw = a.tobytes()
assert len(raw) == 24, f"d bytes = {len(raw)!r}"  # 3 * 8 bytes
b = array.array("d")
b.frombytes(raw)
assert abs(b[1] - 2.2) < 1e-10, f"float frombytes = {b[1]!r}"

print("double_tobytes_frombytes_roundtrip OK")
