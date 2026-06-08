# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "getopt"
# dimension = "behavior"
# case = "double_dash_terminates_options"
# subject = "getopt.getopt"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_getopt.py"
# status = "filled"
# ///
"""getopt.getopt: '--' terminates option processing; everything after it is treated as args"""
import getopt

opts, args = getopt.getopt(['-v', '--', '-h'], 'vh')
assert opts == [('-v', '')], opts
assert args == ['-h'], args
print("double_dash_terminates_options OK")
