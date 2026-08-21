# Atomic 282 pass conformance — argparse module (hasattr
# ArgumentParser) + optparse module (hasattr OptionParser/Option/
# OptionGroup/Values/OptionError/OptionValueError/BadOptionError/
# OptionConflictError/IndentedHelpFormatter/TitledHelpFormatter/
# SUPPRESS_HELP/SUPPRESS_USAGE) + configparser module (hasattr
# ConfigParser) + getpass module (hasattr getpass/getuser +
# getuser returns a non-empty str).
# All asserts match between CPython 3.12 and mamba.
import argparse
import optparse
import configparser
import getpass


_ledger: list[int] = []

# 1) argparse — top-level parser class
assert hasattr(argparse, "ArgumentParser") == True; _ledger.append(1)

# 2) optparse — parser/group/values surface
assert hasattr(optparse, "OptionParser") == True; _ledger.append(1)
assert hasattr(optparse, "Option") == True; _ledger.append(1)
assert hasattr(optparse, "OptionGroup") == True; _ledger.append(1)
assert hasattr(optparse, "Values") == True; _ledger.append(1)

# 3) optparse — exception surface
assert hasattr(optparse, "OptionError") == True; _ledger.append(1)
assert hasattr(optparse, "OptionValueError") == True; _ledger.append(1)
assert hasattr(optparse, "BadOptionError") == True; _ledger.append(1)
assert hasattr(optparse, "OptionConflictError") == True; _ledger.append(1)

# 4) optparse — formatter surface
assert hasattr(optparse, "IndentedHelpFormatter") == True; _ledger.append(1)
assert hasattr(optparse, "TitledHelpFormatter") == True; _ledger.append(1)

# 5) optparse — SUPPRESS_* constants present
assert hasattr(optparse, "SUPPRESS_HELP") == True; _ledger.append(1)
assert hasattr(optparse, "SUPPRESS_USAGE") == True; _ledger.append(1)

# 6) configparser — parser surface
assert hasattr(configparser, "ConfigParser") == True; _ledger.append(1)

# 7) getpass — function surface
assert hasattr(getpass, "getpass") == True; _ledger.append(1)
assert hasattr(getpass, "getuser") == True; _ledger.append(1)

# 8) getpass.getuser() — returns non-empty str
_user = getpass.getuser()
assert isinstance(_user, str) == True; _ledger.append(1)
assert (len(_user) > 0) == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_argparse_optparse_configparser_getpass_value_ops {sum(_ledger)} asserts")
