# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "getopt"
# dimension = "behavior"
# case = "posix_stops_at_first_nonoption"
# subject = "getopt.getopt"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_getopt.py"
# status = "filled"
# ///
"""getopt.getopt: POSIX getopt stops scanning at the first non-option, leaving the rest as args"""
import getopt

opts, args = getopt.getopt(['-v', 'arg', '-h'], 'vh')
assert opts == [('-v', '')], opts
assert args == ['arg', '-h'], args
print("posix_stops_at_first_nonoption OK")
