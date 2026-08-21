# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "getopt"
# dimension = "behavior"
# case = "long_option_flag"
# subject = "getopt.getopt"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_getopt.py"
# status = "filled"
# ///
"""getopt.getopt: a long flag '--help' (no '=') parses to [('--help', '')] with empty value"""
import getopt

opts, args = getopt.getopt(['--help'], '', ['help'])
assert opts == [('--help', '')], opts
assert args == [], args
print("long_option_flag OK")
