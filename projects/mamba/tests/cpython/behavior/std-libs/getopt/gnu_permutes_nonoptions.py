# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "getopt"
# dimension = "behavior"
# case = "gnu_permutes_nonoptions"
# subject = "getopt.gnu_getopt"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_getopt.py"
# status = "filled"
# ///
"""getopt.gnu_getopt: GNU getopt permutes options past intervening non-options, collecting them as args"""
import getopt

opts, args = getopt.gnu_getopt(['-v', 'arg', '-h'], 'vh')
assert opts == [('-v', ''), ('-h', '')], opts
assert args == ['arg'], args
print("gnu_permutes_nonoptions OK")
