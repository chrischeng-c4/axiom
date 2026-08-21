# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_pep614_decorator"
# subject = "cpython321.lang_pep614_decorator"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/lang_pep614_decorator.py"
# status = "filled"
# ///
"""cpython321.lang_pep614_decorator: execute CPython 3.12 seed lang_pep614_decorator"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for PEP 614 — relaxed decorator
# grammar (Py 3.9+).
# Surface: arbitrary expressions allowed in decorator position, not
# just dotted names. Covers paren-wrapped expression (@(expr)) form;
# the subscript-decorator form (@registry[0]) currently mis-binds
# the wrapped function on mamba and is tracked as a separate gap.
def loud(f):
    def w(*args, **kw):
        return f(*args, **kw).upper()
    return w

def quiet(f):
    return f

@(quiet)
def passthrough(x):
    return x

@(loud)
def shout(msg):
    return msg

_ledger: list[int] = []
# Paren-wrapped identity decorator preserves str behaviour
assert passthrough("abc") == "abc"; _ledger.append(1)
assert passthrough("") == ""; _ledger.append(1)
# Paren-wrapped wrapper decorator transforms str output
assert shout("hello") == "HELLO"; _ledger.append(1)
assert shout("World") == "WORLD"; _ledger.append(1)
assert shout("") == ""; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: lang_pep614_decorator {sum(_ledger)} asserts")
