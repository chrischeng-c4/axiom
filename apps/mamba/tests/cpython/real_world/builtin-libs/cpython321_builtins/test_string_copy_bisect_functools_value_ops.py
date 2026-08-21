# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "cpython321_builtins"
# dimension = "real_world"
# case = "test_string_copy_bisect_functools_value_ops"
# subject = "cpython321.test_string_copy_bisect_functools_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_string_copy_bisect_functools_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_string_copy_bisect_functools_value_ops: execute CPython 3.12 seed test_string_copy_bisect_functools_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Atomic 297 pass conformance — string module (hasattr ascii_lower/
# upper/letters/digits/hexdigits/octdigits/punctuation/whitespace/
# Template/Formatter/capwords + constant values + capwords) +
# textwrap module (hasattr wrap/fill/dedent/indent/shorten + dedent
# + indent) + shlex module (hasattr split/join/quote + split/quote/
# join) + difflib module (hasattr SequenceMatcher/unified_diff/get_
# close_matches + get_close_matches) + copy module (hasattr copy/
# deepcopy/Error + copy/deepcopy contracts) + bisect module (hasattr
# bisect/bisect_left/bisect_right/insort/insort_left/insort_right +
# bisect_left/right values) + functools module (hasattr reduce/
# partial/lru_cache/cache/cached_property/wraps/update_wrapper/cmp_
# to_key/singledispatch/singledispatchmethod/partialmethod/total_
# ordering + reduce).
# All asserts match between CPython 3.12 and mamba.
import string
import textwrap
import shlex
import difflib
import copy
import bisect
import functools


_ledger: list[int] = []

# 1) string — hasattr core surface (conformant subset)
assert hasattr(string, "ascii_lowercase") == True; _ledger.append(1)
assert hasattr(string, "ascii_uppercase") == True; _ledger.append(1)
assert hasattr(string, "ascii_letters") == True; _ledger.append(1)
assert hasattr(string, "digits") == True; _ledger.append(1)
assert hasattr(string, "hexdigits") == True; _ledger.append(1)
assert hasattr(string, "octdigits") == True; _ledger.append(1)
assert hasattr(string, "punctuation") == True; _ledger.append(1)
assert hasattr(string, "whitespace") == True; _ledger.append(1)
assert hasattr(string, "Template") == True; _ledger.append(1)
assert hasattr(string, "Formatter") == True; _ledger.append(1)
assert hasattr(string, "capwords") == True; _ledger.append(1)

# 2) string — constant values
assert string.ascii_lowercase == "abcdefghijklmnopqrstuvwxyz"; _ledger.append(1)
assert string.ascii_uppercase == "ABCDEFGHIJKLMNOPQRSTUVWXYZ"; _ledger.append(1)
assert string.digits == "0123456789"; _ledger.append(1)
assert string.hexdigits == "0123456789abcdefABCDEF"; _ledger.append(1)
assert string.octdigits == "01234567"; _ledger.append(1)
assert string.capwords("hello world") == "Hello World"; _ledger.append(1)

# 3) textwrap — hasattr core surface (conformant subset)
assert hasattr(textwrap, "wrap") == True; _ledger.append(1)
assert hasattr(textwrap, "fill") == True; _ledger.append(1)
assert hasattr(textwrap, "dedent") == True; _ledger.append(1)
assert hasattr(textwrap, "indent") == True; _ledger.append(1)
assert hasattr(textwrap, "shorten") == True; _ledger.append(1)

# 4) textwrap — value contracts (conformant subset)
assert textwrap.dedent("    a\n    b") == "a\nb"; _ledger.append(1)
assert textwrap.indent("a\nb", "  ") == "  a\n  b"; _ledger.append(1)

# 5) shlex — hasattr core surface (conformant subset)
assert hasattr(shlex, "split") == True; _ledger.append(1)
assert hasattr(shlex, "join") == True; _ledger.append(1)
assert hasattr(shlex, "quote") == True; _ledger.append(1)

# 6) shlex — value contracts
assert shlex.split("a b c") == ["a", "b", "c"]; _ledger.append(1)
assert shlex.split('a "b c"') == ["a", "b c"]; _ledger.append(1)
assert shlex.quote("a b") == "'a b'"; _ledger.append(1)
assert shlex.quote("a") == "a"; _ledger.append(1)
assert shlex.join(["a", "b"]) == "a b"; _ledger.append(1)

# 7) difflib — hasattr core surface (conformant subset)
assert hasattr(difflib, "SequenceMatcher") == True; _ledger.append(1)
assert hasattr(difflib, "unified_diff") == True; _ledger.append(1)
assert hasattr(difflib, "get_close_matches") == True; _ledger.append(1)

# 8) difflib — value contracts
assert difflib.get_close_matches("app", ["ape", "apple", "aple"]) == ["apple", "ape"]; _ledger.append(1)

# 9) copy — hasattr core surface
assert hasattr(copy, "copy") == True; _ledger.append(1)
assert hasattr(copy, "deepcopy") == True; _ledger.append(1)
assert hasattr(copy, "Error") == True; _ledger.append(1)

# 10) copy — value contracts
assert copy.copy([1, 2, 3]) == [1, 2, 3]; _ledger.append(1)
assert copy.deepcopy([[1, 2], [3, 4]]) == [[1, 2], [3, 4]]; _ledger.append(1)
assert copy.deepcopy({"a": [1, 2]}) == {"a": [1, 2]}; _ledger.append(1)

# 11) bisect — hasattr core surface
assert hasattr(bisect, "bisect") == True; _ledger.append(1)
assert hasattr(bisect, "bisect_left") == True; _ledger.append(1)
assert hasattr(bisect, "bisect_right") == True; _ledger.append(1)
assert hasattr(bisect, "insort") == True; _ledger.append(1)
assert hasattr(bisect, "insort_left") == True; _ledger.append(1)
assert hasattr(bisect, "insort_right") == True; _ledger.append(1)

# 12) bisect — value contracts
assert bisect.bisect_left([1, 3, 5], 3) == 1; _ledger.append(1)
assert bisect.bisect_right([1, 3, 5], 3) == 2; _ledger.append(1)
assert bisect.bisect([1, 3, 5], 4) == 2; _ledger.append(1)

# 13) functools — hasattr core surface
assert hasattr(functools, "reduce") == True; _ledger.append(1)
assert hasattr(functools, "partial") == True; _ledger.append(1)
assert hasattr(functools, "lru_cache") == True; _ledger.append(1)
assert hasattr(functools, "cache") == True; _ledger.append(1)
assert hasattr(functools, "cached_property") == True; _ledger.append(1)
assert hasattr(functools, "wraps") == True; _ledger.append(1)
assert hasattr(functools, "update_wrapper") == True; _ledger.append(1)
assert hasattr(functools, "cmp_to_key") == True; _ledger.append(1)
assert hasattr(functools, "singledispatch") == True; _ledger.append(1)
assert hasattr(functools, "singledispatchmethod") == True; _ledger.append(1)
assert hasattr(functools, "partialmethod") == True; _ledger.append(1)
assert hasattr(functools, "total_ordering") == True; _ledger.append(1)

# 14) functools — value contracts
assert functools.reduce(lambda a, b: a + b, [1, 2, 3]) == 6; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_string_copy_bisect_functools_value_ops {sum(_ledger)} asserts")
