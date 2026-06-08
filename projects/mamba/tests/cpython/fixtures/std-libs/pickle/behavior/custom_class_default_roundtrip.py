# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "behavior"
# case = "custom_class_default_roundtrip"
# subject = "pickle.loads"
# kind = "semantic"
# xfail = "pickle shim serializes Instance objects to the 'N' sentinel; default user-class pickling is unsupported (src/runtime/stdlib/pickle_mod.rs:220)"
# mem_carveout = ""
# source = "Lib/test/test_pickle.py"
# status = "filled"
# ///
"""pickle.loads: a plain user class with instance attributes round-trips via default pickling: the reconstructed object is an instance of the class and compares equal by attributes"""
import pickle


class Point:
    def __init__(self, x, y):
        self.x = x
        self.y = y

    def __eq__(self, other):
        return isinstance(other, Point) and self.x == other.x and self.y == other.y


p = Point(3, 4)
rt = pickle.loads(pickle.dumps(p))
assert isinstance(rt, Point), f"custom class type = {type(rt)!r}"
assert rt == p, "custom class equality"

print("custom_class_default_roundtrip OK")
