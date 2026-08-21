# Operational AssertionPass seed for the value contract of four
# pure-utility stdlib modules used by every CLI front end:
# `shlex` (POSIX shell-style tokenizer / quote / join), `getopt`
# (the long-standing short / long option parser), the
# documented "extras" surface of `itertools` (islice / takewhile /
# dropwhile / filterfalse / compress / starmap / accumulate /
# pairwise) — the lazy combinators that anchor every functional
# pipeline — and the per-level `logging` integer constants
# (DEBUG / INFO / WARNING / ERROR / CRITICAL) used by every
# observability layer. No fixture coverage yet for shlex / getopt /
# itertools-extras / logging-level-constants at this level of
# detail.
#
# The matching subset between mamba and CPython is the byte-exact
# return-value layer: shlex.split on unquoted input + shlex.quote
# / join exactly reproduce the documented tokens; getopt.getopt
# returns the documented (opts, args) pair shape for both short
# and long-option syntaxes; the eight itertools extras each
# return the documented sequence; every documented `logging.*`
# integer level constant has the documented value; platform.system /
# python_version / machine all return `str` (without checking the
# implementation-specific value).
#
# Surface in this fixture:
#   • shlex.split("hello world") == ["hello", "world"];
#   • shlex.split("a b c") == ["a", "b", "c"];
#   • shlex.split("a") == ["a"];
#   • shlex.quote("hello world") == "'hello world'";
#   • shlex.quote("simple") == "simple";
#   • shlex.join(["a", "b c", "d"]) == "a 'b c' d";
#   • getopt.getopt(["-a", "-bvalue", "rest"], "ab:") ==
#     ([("-a",""), ("-b","value")], ["rest"]);
#   • getopt.getopt(["--name=alice", "rest"], "", ["name="]) ==
#     ([("--name","alice")], ["rest"]);
#   • itertools.islice([1,2,3,4,5], 2, 4) -> [3, 4];
#   • itertools.takewhile(lambda x: x < 3, [1,2,3,4,1]) -> [1, 2];
#   • itertools.dropwhile(lambda x: x < 3, [1,2,3,4,1]) -> [3, 4, 1];
#   • itertools.filterfalse(lambda x: x%2, [1,2,3,4,5]) -> [2, 4];
#   • itertools.compress([1,2,3,4], [1,0,1,0]) -> [1, 3];
#   • itertools.starmap(lambda a, b: a+b, [(1,2),(3,4)]) -> [3, 7];
#   • itertools.accumulate([1,2,3,4]) -> [1, 3, 6, 10];
#   • itertools.pairwise([1,2,3,4]) -> [(1,2),(2,3),(3,4)];
#   • logging.DEBUG == 10 / INFO == 20 / WARNING == 30 /
#     ERROR == 40 / CRITICAL == 50 (documented level integers);
#   • platform.system() / python_version() / machine() each return
#     a `str`.
#
# Behavioral edges that DIVERGE on mamba (ipaddress entire surface,
# numbers ABC class identity + isinstance, shlex.split honoring
# quoted tokens, platform.architecture / python_implementation /
# uname, itertools.chain.from_iterable, logging.NOTSET integer
# value, logging.getLevelName, logging.getLogger returning a Logger
# instance, logging.Logger / Handler / StreamHandler class identity)
# are covered in `lang_ipaddress_numbers_logging_logger_silent`.
import shlex
import getopt
import itertools
import logging
import platform

_ledger: list[int] = []

# 1) shlex.split — unquoted input
assert shlex.split("hello world") == ["hello", "world"]; _ledger.append(1)
assert shlex.split("a b c") == ["a", "b", "c"]; _ledger.append(1)
assert shlex.split("a") == ["a"]; _ledger.append(1)

# 2) shlex.quote — quoting strings that need escaping
assert shlex.quote("hello world") == "'hello world'"; _ledger.append(1)
assert shlex.quote("simple") == "simple"; _ledger.append(1)

# 3) shlex.join — re-joining a token list with proper quoting
assert shlex.join(["a", "b c", "d"]) == "a 'b c' d"; _ledger.append(1)

# 4) getopt.getopt — short-option syntax
_opts, _args = getopt.getopt(["-a", "-bvalue", "rest"], "ab:")
assert _opts == [("-a", ""), ("-b", "value")]; _ledger.append(1)
assert _args == ["rest"]; _ledger.append(1)

# 5) getopt.getopt — long-option syntax
_lopts, _largs = getopt.getopt(["--name=alice", "rest"], "", ["name="])
assert _lopts == [("--name", "alice")]; _ledger.append(1)
assert _largs == ["rest"]; _ledger.append(1)

# 6) itertools.islice — start / stop slice
assert list(itertools.islice([1, 2, 3, 4, 5], 2, 4)) == [3, 4]; _ledger.append(1)
assert list(itertools.islice([1, 2, 3, 4, 5], 3)) == [1, 2, 3]; _ledger.append(1)

# 7) itertools.takewhile / dropwhile — predicate-based partition
assert list(itertools.takewhile(lambda x: x < 3, [1, 2, 3, 4, 1])) == [1, 2]; _ledger.append(1)
assert list(itertools.dropwhile(lambda x: x < 3, [1, 2, 3, 4, 1])) == [3, 4, 1]; _ledger.append(1)

# 8) itertools.filterfalse — negated filter
assert list(itertools.filterfalse(lambda x: x % 2, [1, 2, 3, 4, 5])) == [2, 4]; _ledger.append(1)

# 9) itertools.compress — selector-mask gating
assert list(itertools.compress([1, 2, 3, 4], [1, 0, 1, 0])) == [1, 3]; _ledger.append(1)

# 10) itertools.starmap — apply function to tuple-arg pairs
assert list(itertools.starmap(lambda a, b: a + b, [(1, 2), (3, 4)])) == [3, 7]; _ledger.append(1)

# 11) itertools.accumulate — running totals
assert list(itertools.accumulate([1, 2, 3, 4])) == [1, 3, 6, 10]; _ledger.append(1)

# 12) itertools.pairwise — adjacent (i, i+1) pairs
assert list(itertools.pairwise([1, 2, 3, 4])) == [(1, 2), (2, 3), (3, 4)]; _ledger.append(1)

# 13) logging — documented per-level integer constants
assert logging.DEBUG == 10; _ledger.append(1)
assert logging.INFO == 20; _ledger.append(1)
assert logging.WARNING == 30; _ledger.append(1)
assert logging.ERROR == 40; _ledger.append(1)
assert logging.CRITICAL == 50; _ledger.append(1)

# 14) platform.system / python_version / machine — return str
assert isinstance(platform.system(), str); _ledger.append(1)
assert isinstance(platform.python_version(), str); _ledger.append(1)
assert isinstance(platform.machine(), str); _ledger.append(1)

# 15) hasattr surface — module-level helpers
assert hasattr(shlex, "split"); _ledger.append(1)
assert hasattr(shlex, "quote"); _ledger.append(1)
assert hasattr(shlex, "join"); _ledger.append(1)
assert hasattr(getopt, "getopt"); _ledger.append(1)
assert hasattr(itertools, "islice"); _ledger.append(1)
assert hasattr(itertools, "takewhile"); _ledger.append(1)
assert hasattr(itertools, "accumulate"); _ledger.append(1)
assert hasattr(logging, "DEBUG"); _ledger.append(1)
assert hasattr(logging, "INFO"); _ledger.append(1)
assert hasattr(platform, "system"); _ledger.append(1)

# NB: ipaddress entire surface, numbers ABC class identity +
# isinstance, shlex.split honoring quoted tokens, platform.
# architecture / python_implementation / uname, itertools.chain.
# from_iterable, logging.NOTSET integer value, logging.getLevelName,
# logging.getLogger returning a Logger instance, logging.Logger /
# Handler / StreamHandler class identity all DIVERGE on mamba —
# moved to the divergence-spec fixture.

print(f"MAMBA_ASSERTION_PASS: test_shlex_getopt_itertools_logging_ops {sum(_ledger)} asserts")
