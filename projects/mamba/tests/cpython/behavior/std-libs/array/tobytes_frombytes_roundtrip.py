# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "behavior"
# case = "tobytes_frombytes_roundtrip"
# subject = "array.array"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_array.py"
# status = "filled"
# ///
"""array.array: tobytes()/frombytes() round-trip preserves int values; 3 int32 elements serialize to 12 bytes"""
import array

a = array.array("i", [10, 20, 30])
raw = a.tobytes()
assert isinstance(raw, bytes), f"tobytes type = {type(raw)!r}"
assert len(raw) == 12, f"bytes len = {len(raw)!r}"  # 3 * 4 bytes
b = array.array("i")
b.frombytes(raw)
assert b.tolist() == [10, 20, 30], f"frombytes = {b.tolist()!r}"

print("tobytes_frombytes_roundtrip OK")
