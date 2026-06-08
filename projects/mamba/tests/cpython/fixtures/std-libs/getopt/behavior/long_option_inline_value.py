# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "getopt"
# dimension = "behavior"
# case = "long_option_inline_value"
# subject = "getopt.getopt"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_getopt.py"
# status = "filled"
# ///
"""getopt.getopt: a long option with inline '--output=foo' splits name and value -> [('--output', 'foo')]"""
import getopt

opts, args = getopt.getopt(['--output=foo'], '', ['output='])
assert opts == [('--output', 'foo')], opts
assert args == [], args
print("long_option_inline_value OK")
