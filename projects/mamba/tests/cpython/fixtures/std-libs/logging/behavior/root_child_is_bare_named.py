# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "behavior"
# case = "root_child_is_bare_named"
# subject = "logging.Logger"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""logging.Logger: a child of the root logger has the bare name with no leading dot: root.getChild('x') is getLogger('x')"""
import logging

root = logging.getLogger()
xyz = root.getChild("hier_top")
assert xyz is logging.getLogger("hier_top"), "root child is bare-named"
print("root_child_is_bare_named OK")
