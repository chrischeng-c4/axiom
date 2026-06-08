# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "behavior"
# case = "removedirs_prunes_empty_parents"
# subject = "os.removedirs"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.removedirs: os.removedirs deletes the leaf and prunes now-empty parents up the chain, but stops at the first non-empty parent"""
import os
import tempfile

with tempfile.TemporaryDirectory() as base:
    # removedirs deletes leaf and prunes now-empty parents up the chain.
    tree = os.path.join(base, "a", "b", "c")
    os.makedirs(tree)
    os.removedirs(tree)
    assert not os.path.exists(os.path.join(base, "a")), "empty parents pruned"

    # removedirs stops at the first non-empty parent.
    kept = os.path.join(base, "x", "y")
    os.makedirs(os.path.join(kept, "z"))
    with open(os.path.join(kept, "sibling.txt"), "w", encoding="utf-8") as f:
        f.write("keep")
    os.removedirs(os.path.join(kept, "z"))
    assert not os.path.exists(os.path.join(kept, "z")), "leaf removed"
    assert os.path.isdir(kept), "non-empty parent kept"
print("removedirs_prunes_empty_parents OK")
