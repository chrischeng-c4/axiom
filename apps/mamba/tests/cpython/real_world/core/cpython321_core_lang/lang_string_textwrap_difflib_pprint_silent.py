# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_string_textwrap_difflib_pprint_silent"
# subject = "cpython321.lang_string_textwrap_difflib_pprint_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_string_textwrap_difflib_pprint_silent.py"
# status = "filled"
# ///
"""cpython321.lang_string_textwrap_difflib_pprint_silent: execute CPython 3.12 seed lang_string_textwrap_difflib_pprint_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass divergence-spec fixture for the silent
# value-contract divergences of `hasattr(string, 'printable')` (the
# documented "string exposes the printable character pool" — mamba
# returns False), `type(string.Template('$name')).__name__ ==
# 'Template'` (the documented "string.Template returns a Template
# instance" — mamba returns 'dict' — constructor degrades to a
# plain dict), `hasattr(textwrap, 'TextWrapper')` (the documented
# "textwrap exposes the TextWrapper class" — mamba returns False),
# `textwrap.fill('aaa bbb ccc ddd', width=8) == 'aaa bbb\\nccc ddd
# '` (the documented "textwrap.fill wraps at width" — mamba returns
# 'aaa bbb ccc ddd' — no width-based wrapping), `textwrap.shorten('
# a b c d', width=5) == '[...]'` (the documented "textwrap.shorten
# truncates with placeholder when over width" — mamba returns
# 'a b c d' — no truncation), `hasattr(shlex, 'shlex')` (the
# documented "shlex exposes the shlex lexer class" — mamba returns
# False), `hasattr(difflib, 'Differ')` (the documented "difflib
# exposes the Differ class" — mamba returns False),
# `type(difflib.SequenceMatcher(None, 'a', 'b')).__name__ ==
# 'SequenceMatcher'` (the documented "SequenceMatcher constructor
# returns a SequenceMatcher instance" — mamba returns 'float' —
# constructor returns a similarity ratio float instead of an
# instance), `hasattr(pprint, 'PrettyPrinter')` (the documented
# "pprint exposes the PrettyPrinter class" — mamba returns False),
# and `pprint.pformat([1, 2]) == '[1, 2]'` (the documented "pformat
# inlines small sequences" — mamba returns '[\\n 1,\\n 2\\n]' —
# forced multi-line layout regardless of width).
# Ten-pack pinned to atomic 297.
#
# Behavioral edges that CONFORM on mamba (string — hasattr ascii_
# lower/upper/letters/digits/hexdigits/octdigits/punctuation/
# whitespace/Template/Formatter/capwords + ascii/hex/oct/digits
# value contracts + capwords. textwrap — hasattr wrap/fill/dedent/
# indent/shorten + dedent + indent. shlex — hasattr split/join/
# quote + split/quote/join contracts. difflib — hasattr
# SequenceMatcher/unified_diff/get_close_matches + get_close_
# matches. copy — hasattr copy/deepcopy/Error + copy/deepcopy
# contracts. bisect — hasattr bisect/bisect_left/right/insort +
# bisect_left/right values. functools — hasattr reduce/partial/
# lru_cache/cache/cached_property/wraps/update_wrapper/cmp_to_key/
# singledispatch/singledispatchmethod/partialmethod/total_ordering
# + reduce) are covered in the matching pass fixture `test_string_
# copy_bisect_functools_value_ops`.
import string
import textwrap
import shlex
import difflib
import pprint


_ledger: list[int] = []

# 1) hasattr(string, 'printable') — printable character pool
#    (mamba: returns False)
assert hasattr(string, "printable") == True; _ledger.append(1)

# 2) type(string.Template('$name')).__name__ == 'Template' — Template instance
#    (mamba: returns 'dict' — constructor degrades to plain dict)
assert type(string.Template("$name")).__name__ == "Template"; _ledger.append(1)

# 3) hasattr(textwrap, 'TextWrapper') — TextWrapper class
#    (mamba: returns False)
assert hasattr(textwrap, "TextWrapper") == True; _ledger.append(1)

# 4) textwrap.fill('aaa bbb ccc ddd', width=8) — width-based wrapping
#    (mamba: returns 'aaa bbb ccc ddd' — no wrapping)
assert textwrap.fill("aaa bbb ccc ddd", width=8) == "aaa bbb\nccc ddd"; _ledger.append(1)

# 5) textwrap.shorten('a b c d', width=5) == '[...]' — width-based truncation
#    (mamba: returns 'a b c d' — no truncation)
assert textwrap.shorten("a b c d", width=5) == "[...]"; _ledger.append(1)

# 6) hasattr(shlex, 'shlex') — shlex lexer class
#    (mamba: returns False)
assert hasattr(shlex, "shlex") == True; _ledger.append(1)

# 7) hasattr(difflib, 'Differ') — Differ class
#    (mamba: returns False)
assert hasattr(difflib, "Differ") == True; _ledger.append(1)

# 8) type(difflib.SequenceMatcher(None, 'a', 'b')).__name__ == 'SequenceMatcher' — SequenceMatcher instance
#    (mamba: returns 'float' — constructor returns similarity ratio float)
assert type(difflib.SequenceMatcher(None, "a", "b")).__name__ == "SequenceMatcher"; _ledger.append(1)

# 9) hasattr(pprint, 'PrettyPrinter') — PrettyPrinter class
#    (mamba: returns False)
assert hasattr(pprint, "PrettyPrinter") == True; _ledger.append(1)

# 10) pprint.pformat([1, 2]) == '[1, 2]' — inlined small-sequence layout
#     (mamba: returns '[\\n 1,\\n 2\\n]' — forced multi-line)
assert pprint.pformat([1, 2]) == "[1, 2]"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_string_textwrap_difflib_pprint_silent {sum(_ledger)} asserts")
