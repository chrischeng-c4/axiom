# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "getopt"
# dimension = "behavior"
# case = "short_option_no_arg"
# subject = "getopt.getopt"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_getopt.py"
# status = "filled"
# ///
"""getopt.getopt: a bare short flag '-v' parses to [('-v', '')] with no remaining args"""
import getopt

opts, args = getopt.getopt(['-v'], 'v')
assert opts == [('-v', '')], opts
assert args == [], args
print("short_option_no_arg OK")
