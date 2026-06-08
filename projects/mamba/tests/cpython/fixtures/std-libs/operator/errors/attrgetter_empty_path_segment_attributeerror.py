# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "errors"
# case = "attrgetter_empty_path_segment_attributeerror"
# subject = "operator.attrgetter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.attrgetter: empty dotted-path segments ('child.' and '.child') are not valid attribute names so attrgetter raises AttributeError"""
import operator


class Node:
    pass


root = Node()
root.child = Node()

for bad in ("child.", ".child"):
    _raised = False
    try:
        operator.attrgetter(bad)(root)
    except AttributeError:
        _raised = True
    assert _raised, f"expected AttributeError for {bad!r}"
print("attrgetter_empty_path_segment_attributeerror OK")
