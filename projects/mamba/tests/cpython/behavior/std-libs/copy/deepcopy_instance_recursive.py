# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "deepcopy_instance_recursive"
# subject = "copy.deepcopy"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
"""copy.deepcopy: deepcopy of a linked-node instance copies the chain into fresh, independent instances"""
import copy


class Node:
    def __init__(self, val, child=None):
        self.val = val
        self.child = child


root = Node(1, Node(2))
deep = copy.deepcopy(root)
assert deep is not root, "deepcopy of the root is a new instance"
assert deep.val == 1, f"root value preserved = {deep.val!r}"
assert deep.child.val == 2, f"child value preserved = {deep.child.val!r}"
assert deep.child is not root.child, "the child node is a fresh instance too"

print("deepcopy_instance_recursive OK")
