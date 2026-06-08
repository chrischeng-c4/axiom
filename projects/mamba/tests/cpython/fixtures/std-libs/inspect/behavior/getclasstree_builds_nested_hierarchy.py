# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "getclasstree_builds_nested_hierarchy"
# subject = "inspect.getclasstree"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.getclasstree: getclasstree() builds a nested inheritance hierarchy with object at the top and subclasses nested under their base"""
import inspect

class A:
    pass

class B(A):
    pass

class C(A):
    pass

tree = inspect.getclasstree([A, B, C])
assert tree[0] == (object, ()), f"tree root = {tree[0]!r}"
under_object = tree[1]
assert (A, (object,)) in under_object, "A nested under object"
under_a = under_object[1]
assert (B, (A,)) in under_a, "B nested under A"
assert (C, (A,)) in under_a, "C nested under A"

print("getclasstree_builds_nested_hierarchy OK")
