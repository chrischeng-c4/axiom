# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "behavior"
# case = "traceback_repr_frame_ordering"
# subject = "tracemalloc.Traceback"
# kind = "semantic"
# xfail = "mamba does not implement the tracemalloc.Traceback class (#1261 long-tail stub batch)"
# mem_carveout = ""
# source = "Lib/test/test_tracemalloc.py"
# status = "filled"
# ///
"""tracemalloc.Traceback: repr(Traceback) lists frames newest-first and only shows total_nframe when an explicit total is supplied"""
import tracemalloc

# Empty traceback.
assert repr(tracemalloc.Traceback(())) == "<Traceback ()>", "empty repr"
assert (
    repr(tracemalloc.Traceback((), 0)) == "<Traceback () total_nframe=0>"
), "empty repr with total"

# Frames render newest-first; constructor input is oldest-first.
frames = (("f1", 1), ("f2", 2))
exp_frames = "(<Frame filename='f2' lineno=2>, <Frame filename='f1' lineno=1>)"
assert repr(tracemalloc.Traceback(frames)) == f"<Traceback {exp_frames}>", "frames repr"
assert (
    repr(tracemalloc.Traceback(frames, 2))
    == f"<Traceback {exp_frames} total_nframe=2>"
), "frames repr with total"

print("traceback_repr_frame_ordering OK")
