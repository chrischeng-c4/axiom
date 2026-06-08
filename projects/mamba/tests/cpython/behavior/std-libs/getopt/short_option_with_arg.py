# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "getopt"
# dimension = "behavior"
# case = "short_option_with_arg"
# subject = "getopt.getopt"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_getopt.py"
# status = "filled"
# ///
"""getopt.getopt: a short option declared 'o:' consumes its following argument -> [('-o', 'foo')]"""
import getopt

opts, args = getopt.getopt(['-o', 'foo'], 'o:')
assert opts == [('-o', 'foo')], opts
assert args == [], args
print("short_option_with_arg OK")
