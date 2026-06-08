# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "behavior"
# case = "pickler_unpickler_roundtrip"
# subject = "pickle.Pickler"
# kind = "semantic"
# xfail = "pickle.Pickler/Unpickler are class shells; construction is out of scope (src/runtime/stdlib/pickle_mod.rs:50-54)"
# mem_carveout = ""
# source = "Lib/test/test_pickle.py"
# status = "filled"
# ///
"""pickle.Pickler: the streaming API round-trips a dict: a Pickler over a BytesIO writes via dump, and an Unpickler over the rewound buffer reconstructs it via load equal to the original"""
import io
import pickle

buf = io.BytesIO()
pickler = pickle.Pickler(buf)
assert hasattr(pickler, "dump"), "Pickler has dump"
pickler.dump({"x": 1})

buf.seek(0)
unpickler = pickle.Unpickler(buf)
assert hasattr(unpickler, "load"), "Unpickler has load"
assert unpickler.load() == {"x": 1}, "Pickler/Unpickler round-trip"

print("pickler_unpickler_roundtrip OK")
