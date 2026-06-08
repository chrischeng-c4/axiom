# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "getopt"
# dimension = "real_world"
# case = "libref_unix_and_long_options"
# subject = "getopt.getopt"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_getopt.py"
# status = "filled"
# ///
"""getopt.getopt: the Library Reference example: parse a Unix-style argv (clustered '-cfoo', separate '-d bar') and a long-option argv ('--condition=foo', '--output-file abc.def') into the documented optlist + trailing args"""
import getopt

# Unix-style options: -a/-b flags, -c takes 'foo' (clustered as -cfoo),
# -d takes 'bar' (separate token); a1/a2 are trailing positional args.
unix_argv = '-a -b -cfoo -d bar a1 a2'.split()
optlist, args = getopt.getopt(unix_argv, 'abc:d:')
assert optlist == [('-a', ''), ('-b', ''), ('-c', 'foo'), ('-d', 'bar')], optlist
assert args == ['a1', 'a2'], args

# Long option names: --condition=foo (inline), --testing (flag),
# --output-file abc.def (separate value), -x flag; a1/a2 trailing.
long_argv = '--condition=foo --testing --output-file abc.def -x a1 a2'.split()
optlist, args = getopt.getopt(long_argv, 'x', ['condition=', 'output-file=', 'testing'])
assert optlist == [
    ('--condition', 'foo'),
    ('--testing', ''),
    ('--output-file', 'abc.def'),
    ('-x', ''),
], optlist
assert args == ['a1', 'a2'], args

print("libref_unix_and_long_options OK")
