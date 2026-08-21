# Operational AssertionPass divergence-spec fixture for the silent
# value-contract divergences of `hasattr(argparse, 'Namespace')` (the
# documented "argparse exposes the Namespace result class" — mamba
# returns False — argparse module is a dict), `hasattr(argparse,
# 'SUPPRESS')` (the documented "argparse exposes the SUPPRESS
# sentinel" — mamba returns False), `argparse.OPTIONAL == '?'` (the
# documented "argparse.OPTIONAL is the '?' nargs sentinel" — mamba
# returns None), `argparse.ZERO_OR_MORE == '*'` (the documented
# "argparse.ZERO_OR_MORE is the '*' nargs sentinel" — mamba returns
# None), `argparse.ONE_OR_MORE == '+'` (the documented "argparse.
# ONE_OR_MORE is the '+' nargs sentinel" — mamba returns None),
# `optparse.SUPPRESS_HELP == 'SUPPRESSHELP'` (the documented
# "optparse SUPPRESS_HELP is the 'SUPPRESSHELP' sentinel" — mamba
# returns 0), `type(optparse.OptionParser).__name__` (the documented
# "OptionParser metatype is 'type'" — mamba returns 'function' —
# constructor-as-function), `hasattr(configparser, 'DEFAULTSECT')`
# (the documented "configparser exposes the DEFAULTSECT constant" —
# mamba returns False), `configparser.DEFAULTSECT == 'DEFAULT'` (the
# documented "DEFAULTSECT is the 'DEFAULT' section name" — mamba
# returns None), and `hasattr(getpass, 'GetPassWarning')` (the
# documented "getpass exposes the GetPassWarning exception" — mamba
# returns False).
# Ten-pack pinned to atomic 282.
#
# Behavioral edges that CONFORM on mamba (argparse — hasattr
# ArgumentParser. optparse — hasattr OptionParser/Option/OptionGroup/
# Values/OptionError/OptionValueError/BadOptionError/
# OptionConflictError/IndentedHelpFormatter/TitledHelpFormatter/
# SUPPRESS_HELP/SUPPRESS_USAGE. configparser — hasattr ConfigParser.
# getpass — hasattr getpass/getuser + getuser returns non-empty str)
# are covered in the matching pass fixture `test_argparse_optparse_
# configparser_getpass_value_ops`.
import argparse
import optparse
import configparser
import getpass


_ledger: list[int] = []

# 1) hasattr(argparse, 'Namespace') — Namespace result class
#    (mamba: returns False — argparse is a dict)
assert hasattr(argparse, "Namespace") == True; _ledger.append(1)

# 2) hasattr(argparse, 'SUPPRESS') — SUPPRESS sentinel
#    (mamba: returns False)
assert hasattr(argparse, "SUPPRESS") == True; _ledger.append(1)

# 3) argparse.OPTIONAL == '?' — '?' nargs sentinel
#    (mamba: returns None)
assert argparse.OPTIONAL == "?"; _ledger.append(1)

# 4) argparse.ZERO_OR_MORE == '*' — '*' nargs sentinel
#    (mamba: returns None)
assert argparse.ZERO_OR_MORE == "*"; _ledger.append(1)

# 5) argparse.ONE_OR_MORE == '+' — '+' nargs sentinel
#    (mamba: returns None)
assert argparse.ONE_OR_MORE == "+"; _ledger.append(1)

# 6) optparse.SUPPRESS_HELP == 'SUPPRESSHELP' — help sentinel
#    (mamba: returns 0)
assert optparse.SUPPRESS_HELP == "SUPPRESSHELP"; _ledger.append(1)

# 7) type(optparse.OptionParser).__name__ == 'type' — OptionParser metatype
#    (mamba: returns 'function' — constructor-as-function)
assert type(optparse.OptionParser).__name__ == "type"; _ledger.append(1)

# 8) hasattr(configparser, 'DEFAULTSECT') — DEFAULTSECT constant
#    (mamba: returns False)
assert hasattr(configparser, "DEFAULTSECT") == True; _ledger.append(1)

# 9) configparser.DEFAULTSECT == 'DEFAULT' — default section name
#    (mamba: returns None)
assert configparser.DEFAULTSECT == "DEFAULT"; _ledger.append(1)

# 10) hasattr(getpass, 'GetPassWarning') — GetPassWarning exception
#     (mamba: returns False)
assert hasattr(getpass, "GetPassWarning") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_argparse_optparse_configparser_getpass_silent {sum(_ledger)} asserts")
