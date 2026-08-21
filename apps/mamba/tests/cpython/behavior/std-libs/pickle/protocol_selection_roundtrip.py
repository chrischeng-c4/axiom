# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "behavior"
# case = "protocol_selection_roundtrip"
# subject = "pickle.dumps"
# kind = "semantic"
# xfail = "pickle shim ignores the protocol kwarg and emits one fixed text format (src/runtime/stdlib/pickle_mod.rs:318)"
# mem_carveout = ""
# source = "Lib/test/test_pickle.py"
# status = "filled"
# ///
"""pickle.dumps: dumping with an explicit protocol (0 ASCII and 2 binary) and loading back reconstructs the original dict equal to itself"""
import pickle

data = {"test": [1, 2, 3]}
for proto in (0, 2):
    blob = pickle.dumps(data, protocol=proto)
    assert isinstance(blob, bytes), f"protocol {proto} returns bytes"
    assert pickle.loads(blob) == data, f"protocol {proto} round-trip"

print("protocol_selection_roundtrip OK")
