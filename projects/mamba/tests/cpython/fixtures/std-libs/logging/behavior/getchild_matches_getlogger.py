# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "behavior"
# case = "getchild_matches_getlogger"
# subject = "logging.Logger"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""logging.Logger: getChild appends to the dotted name; top.getChild('mod') is the same cached instance as getLogger('top.mod'), and chained/dotted-suffix getChild agree"""
import logging

top = logging.getLogger("hier_pkg")
child = top.getChild("mod")
assert child is logging.getLogger("hier_pkg.mod"), "getChild == getLogger(full)"
deep = top.getChild("mod").getChild("sub")
also_deep = top.getChild("mod.sub")
assert deep is logging.getLogger("hier_pkg.mod.sub"), "chained getChild"
assert deep is also_deep, "dotted suffix == chained"
print("getchild_matches_getlogger OK")
