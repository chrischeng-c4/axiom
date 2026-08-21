# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "behavior"
# case = "dumps_returns_bytes_with_protocol_header"
# subject = "pickle.dumps"
# kind = "semantic"
# xfail = "pickle shim emits a non-CPython text format with no b'\\x80' header byte (src/runtime/stdlib/pickle_mod.rs:318)"
# mem_carveout = ""
# source = "Lib/test/test_pickle.py"
# status = "filled"
# ///
"""pickle.dumps: pickle.dumps returns a bytes object whose first byte is the protocol-2+ opcode marker b'\\x80', and loads round-trips the value"""
import pickle

data = {"key": [1, 2, 3], "value": "hello"}
blob = pickle.dumps(data)
assert isinstance(blob, bytes), f"dumps type = {type(blob)!r}"
assert blob[0:1] == b"\x80", f"pickle header = {blob[0:1]!r}"
assert pickle.loads(blob) == data, "round-trip through dumps/loads"

print("dumps_returns_bytes_with_protocol_header OK")
