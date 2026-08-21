# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "behavior"
# case = "getchildren_immediate_only"
# subject = "logging.Logger"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""logging.Logger: getChildren reports only immediate children (direct child listed, grandchild excluded) and a leaf logger has an empty getChildren set"""
import logging

a = logging.getLogger("hier_tree")
b = logging.getLogger("hier_tree.b")
_grand = logging.getLogger("hier_tree.b.c")
assert b in a.getChildren(), "direct child listed"
direct = {lg.name for lg in a.getChildren()}
assert "hier_tree.b.c" not in direct, "grandchild not listed"
assert _grand.getChildren() == set(), "leaf has empty getChildren"
print("getchildren_immediate_only OK")
