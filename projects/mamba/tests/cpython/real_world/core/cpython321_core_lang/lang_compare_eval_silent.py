# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_compare_eval_silent"
# subject = "cpython321.lang_compare_eval_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_compare_eval_silent.py"
# status = "filled"
# ///
"""cpython321.lang_compare_eval_silent: execute CPython 3.12 seed lang_compare_eval_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass divergence-spec fixture for the silent
# value-contract divergences of `eval("1 < 'a'")` (the documented
# "int < str raises TypeError" — mamba returns False instead of
# raising), `eval("'a' < 1")` (the documented "str < int raises
# TypeError" — mamba returns False instead of raising), `eval("1 <
# None")` (the documented "int < None raises TypeError" — mamba
# returns False instead of raising), `eval("None < 1")` (the
# documented "None < int raises TypeError" — mamba returns False
# instead of raising), `eval("1 <= 'a'")` (the documented "int <= str
# raises TypeError" — mamba returns False instead of raising),
# `eval("[1] < (1,)")` (the documented "list < tuple raises
# TypeError" — mamba returns False instead of raising), `eval("(1,)
# < [1]")` (the documented "tuple < list raises TypeError" — mamba
# returns False instead of raising), `eval("[1] < 'a'")` (the
# documented "list < str raises TypeError" — mamba returns False
# instead of raising), `eval("b'a' < 'a'")` (the documented "bytes <
# str raises TypeError" — mamba returns False instead of raising),
# and `eval("{1} < [1]")` (the documented "set < list raises
# TypeError when list is on the right — mamba returns False instead
# of raising").
# Ten-pack pinned to atomic 321.
#
# Behavioral edges that CONFORM on mamba (same-type ordering on int/
# float/str/list/tuple. Cross-type equality returning False without
# raising. Builtins len/abs/min/max/sum/sorted/reversed/any/all/
# enumerate/zip/map/filter/range/round/divmod/hex/oct/bin/chr/ord/
# repr/str/isinstance/issubclass) are covered in the matching pass
# fixture `test_lang_compare_builtin_value_ops`.


_ledger: list[int] = []

# 1) eval("1 < 'a'") — int < str raises TypeError
#    (mamba: returns False silently)
try:
    eval("1 < 'a'")
    raise AssertionError("expected TypeError")
except TypeError:
    _ledger.append(1)

# 2) eval("'a' < 1") — str < int raises TypeError
#    (mamba: returns False silently)
try:
    eval("'a' < 1")
    raise AssertionError("expected TypeError")
except TypeError:
    _ledger.append(1)

# 3) eval("1 < None") — int < None raises TypeError
#    (mamba: returns False silently)
try:
    eval("1 < None")
    raise AssertionError("expected TypeError")
except TypeError:
    _ledger.append(1)

# 4) eval("None < 1") — None < int raises TypeError
#    (mamba: returns False silently)
try:
    eval("None < 1")
    raise AssertionError("expected TypeError")
except TypeError:
    _ledger.append(1)

# 5) eval("1 <= 'a'") — int <= str raises TypeError
#    (mamba: returns False silently)
try:
    eval("1 <= 'a'")
    raise AssertionError("expected TypeError")
except TypeError:
    _ledger.append(1)

# 6) eval("[1] < (1,)") — list < tuple raises TypeError
#    (mamba: returns False silently)
try:
    eval("[1] < (1,)")
    raise AssertionError("expected TypeError")
except TypeError:
    _ledger.append(1)

# 7) eval("(1,) < [1]") — tuple < list raises TypeError
#    (mamba: returns False silently)
try:
    eval("(1,) < [1]")
    raise AssertionError("expected TypeError")
except TypeError:
    _ledger.append(1)

# 8) eval("[1] < 'a'") — list < str raises TypeError
#    (mamba: returns False silently)
try:
    eval("[1] < 'a'")
    raise AssertionError("expected TypeError")
except TypeError:
    _ledger.append(1)

# 9) eval("b'a' < 'a'") — bytes < str raises TypeError
#    (mamba: returns False silently)
try:
    eval("b'a' < 'a'")
    raise AssertionError("expected TypeError")
except TypeError:
    _ledger.append(1)

# 10) eval("{1} < [1]") — set < list raises TypeError
#     (mamba: returns False silently)
try:
    eval("{1} < [1]")
    raise AssertionError("expected TypeError")
except TypeError:
    _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_compare_eval_silent {sum(_ledger)} asserts")
