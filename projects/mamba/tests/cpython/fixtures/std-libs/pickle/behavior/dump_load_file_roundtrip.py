# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "behavior"
# case = "dump_load_file_roundtrip"
# subject = "pickle.dump"
# kind = "semantic"
# xfail = "pickle.dump is a stub that discards its output and pickle.load returns None (src/runtime/stdlib/pickle_mod.rs:346-353)"
# mem_carveout = ""
# source = "Lib/test/test_pickle.py"
# status = "filled"
# ///
"""pickle.dump: pickle.dump writes a value into a BytesIO file object and pickle.load reads it back from the rewound buffer equal to the original"""
import io
import pickle

data = {"key": [1, 2, 3], "value": "hello"}
buf = io.BytesIO()
pickle.dump(data, buf)
buf.seek(0)
loaded = pickle.load(buf)
assert loaded == data, f"dump/load file round-trip = {loaded!r}"

print("dump_load_file_roundtrip OK")
