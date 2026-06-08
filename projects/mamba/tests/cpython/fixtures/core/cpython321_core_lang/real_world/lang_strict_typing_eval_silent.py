# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_strict_typing_eval_silent"
# subject = "cpython321.lang_strict_typing_eval_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_strict_typing_eval_silent.py"
# status = "filled"
# ///
"""cpython321.lang_strict_typing_eval_silent: execute CPython 3.12 seed lang_strict_typing_eval_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass divergence-spec fixture for the silent
# value-contract divergences of `eval("1 + 'a'")` (the documented
# "int + str raises TypeError" — mamba returns None instead of
# raising), `eval("'a' + 1")` (the documented "str + int raises
# TypeError" — mamba returns None instead of raising), `eval("'a' *
# 2.5")` (the documented "str * float raises TypeError" — mamba
# returns None instead of raising), `eval("[1] + (2,)")` (the
# documented "list + tuple raises TypeError" — mamba returns None
# instead of raising), `eval("(1,) + [2]")` (the documented "tuple +
# list raises TypeError" — mamba returns None instead of raising),
# `eval("[1] * 2.0")` (the documented "list * float raises TypeError"
# — mamba returns None instead of raising), `eval("True + 'a'")` (the
# documented "bool + str raises TypeError" — mamba returns None
# instead of raising), `eval("{1: 'a'} + {2: 'b'}")` (the documented
# "dict + dict raises TypeError; '+' is not defined on dict" — mamba
# returns None instead of raising), `eval("-'a'")` (the documented
# "unary minus on str raises TypeError" — mamba returns None instead
# of raising), and `eval("~'a'")` (the documented "unary ~ on str
# raises TypeError" — mamba returns None instead of raising).
# Ten-pack pinned to atomic 320.
#
# Behavioral edges that CONFORM on mamba (eval() with same-type ops:
# eval("1 + 1.0"), eval("'a' + 'b'"), eval("[1] + [2]"), eval("(1,) +
# (2,)"). Same-type int/float arithmetic. str + str concatenation.
# str * int repeat. list/tuple same-type concat + repeat. set | / &
# / - / ^ ops. dict | merge. string .upper/.strip/.split/.join/
# .replace/.startswith. int()/float()/bool() coercion. unary +/-/~
# on numerics) are covered in the matching pass fixture
# `test_lang_arith_string_concat_value_ops`.


_ledger: list[int] = []

# 1) eval("1 + 'a'") — int + str raises TypeError
#    (mamba: returns None silently)
try:
    eval("1 + 'a'")
    raise AssertionError("expected TypeError")
except TypeError:
    _ledger.append(1)

# 2) eval("'a' + 1") — str + int raises TypeError
#    (mamba: returns None silently)
try:
    eval("'a' + 1")
    raise AssertionError("expected TypeError")
except TypeError:
    _ledger.append(1)

# 3) eval("'a' * 2.5") — str * float raises TypeError
#    (mamba: returns None silently)
try:
    eval("'a' * 2.5")
    raise AssertionError("expected TypeError")
except TypeError:
    _ledger.append(1)

# 4) eval("[1] + (2,)") — list + tuple raises TypeError
#    (mamba: returns None silently)
try:
    eval("[1] + (2,)")
    raise AssertionError("expected TypeError")
except TypeError:
    _ledger.append(1)

# 5) eval("(1,) + [2]") — tuple + list raises TypeError
#    (mamba: returns None silently)
try:
    eval("(1,) + [2]")
    raise AssertionError("expected TypeError")
except TypeError:
    _ledger.append(1)

# 6) eval("[1] * 2.0") — list * float raises TypeError
#    (mamba: returns None silently)
try:
    eval("[1] * 2.0")
    raise AssertionError("expected TypeError")
except TypeError:
    _ledger.append(1)

# 7) eval("True + 'a'") — bool + str raises TypeError
#    (mamba: returns None silently)
try:
    eval("True + 'a'")
    raise AssertionError("expected TypeError")
except TypeError:
    _ledger.append(1)

# 8) eval("{1: 'a'} + {2: 'b'}") — dict + dict raises TypeError
#    (mamba: returns None silently)
try:
    eval("{1: 'a'} + {2: 'b'}")
    raise AssertionError("expected TypeError")
except TypeError:
    _ledger.append(1)

# 9) eval("-'a'") — unary minus on str raises TypeError
#    (mamba: returns None silently)
try:
    eval("-'a'")
    raise AssertionError("expected TypeError")
except TypeError:
    _ledger.append(1)

# 10) eval("~'a'") — unary ~ on str raises TypeError
#     (mamba: returns None silently)
try:
    eval("~'a'")
    raise AssertionError("expected TypeError")
except TypeError:
    _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_strict_typing_eval_silent {sum(_ledger)} asserts")
