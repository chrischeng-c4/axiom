# Operational AssertionPass seed for getopt short/long option parsing.
# Surface: `getopt.getopt(argv, shortspec, longspec=[])` parses POSIX-
# style short and GNU-style long options. `shortspec` is a string of
# option letters with `:` marking ones that take an argument
# (`"ab"` ↔ `-a`/`-b` flags; `"x:"` ↔ `-x VAL`). Short flags may be
# combined (`-ab` parses to two separate flags). `longspec` is a list
# of long-option names with `=` suffix marking ones that take an arg
# (`["alpha"]` ↔ `--alpha`; `["name="]` ↔ `--name=VAL` or
# `--name VAL`). The function returns `(opts, args)` where `opts` is
# a list of `(option, value)` 2-tuples and `args` is the leftover
# positional arguments. A literal `--` argument terminates option
# parsing — everything after lands in `args`. Unknown options or
# missing required arguments raise `getopt.GetoptError`.
# `getopt.gnu_getopt` is the GNU variant that allows interleaving
# positional arguments and options (POSIX getopt stops at the first
# positional; GNU re-orders so all options come first, then args).
import getopt
_ledger: list[int] = []

# Basic short flags
opts, args = getopt.getopt(["-a", "-b"], "ab")
assert opts == [("-a", ""), ("-b", "")]; _ledger.append(1)
assert args == []; _ledger.append(1)

# Short option with argument
opts, args = getopt.getopt(["-x", "val"], "x:")
assert opts == [("-x", "val")]; _ledger.append(1)

# Positional after options
opts, args = getopt.getopt(["-a", "pos1", "pos2"], "a")
assert opts == [("-a", "")]; _ledger.append(1)
assert args == ["pos1", "pos2"]; _ledger.append(1)

# Long flag alone
opts, args = getopt.getopt(["--alpha"], "", ["alpha"])
assert opts == [("--alpha", "")]; _ledger.append(1)

# Long option with = separator
opts, args = getopt.getopt(["--name=Alice"], "", ["name="])
assert opts == [("--name", "Alice")]; _ledger.append(1)

# Long option with space-separated argument
opts, args = getopt.getopt(["--name", "Bob"], "", ["name="])
assert opts == [("--name", "Bob")]; _ledger.append(1)

# Mixed short and long
opts, args = getopt.getopt(["-a", "--beta"], "a", ["beta"])
assert opts == [("-a", ""), ("--beta", "")]; _ledger.append(1)

# Unknown short raises GetoptError
try:
    getopt.getopt(["-z"], "a")
    assert False; _ledger.append(1)
except getopt.GetoptError:
    _ledger.append(1)

# Missing required argument raises GetoptError
try:
    getopt.getopt(["-x"], "x:")
    assert False; _ledger.append(1)
except getopt.GetoptError:
    _ledger.append(1)

# Combined short flags `-ab` parses to two flags
opts, args = getopt.getopt(["-ab"], "ab")
assert opts == [("-a", ""), ("-b", "")]; _ledger.append(1)

# `--` terminator stops option parsing
opts, args = getopt.getopt(["-a", "--", "-b"], "ab")
assert opts == [("-a", "")]; _ledger.append(1)
assert args == ["-b"]; _ledger.append(1)

# Empty argv
opts, args = getopt.getopt([], "ab")
assert opts == []; _ledger.append(1)
assert args == []; _ledger.append(1)

# gnu_getopt re-orders positionals after options
opts, args = getopt.gnu_getopt(["pos", "-a", "more"], "a")
assert opts == [("-a", "")]; _ledger.append(1)
assert args == ["pos", "more"]; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_getopt_short_long_ops {sum(_ledger)} asserts")
