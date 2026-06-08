# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_raise_exception_chain"
# subject = "cpython321.lang_raise_exception_chain"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/lang_raise_exception_chain.py"
# status = "filled"
# ///
"""cpython321.lang_raise_exception_chain: execute CPython 3.12 seed lang_raise_exception_chain"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for raise/except surfaces beyond
# basic try/except in test_exception_ops + lang_try_else_finally.
# Surface: `raise X("msg")` with str(e), custom Exception subclass
# with super().__init__, `raise X from Y` setting __cause__, implicit
# __context__ on raise inside except, `assert False, msg` producing
# AssertionError, bare `raise` re-throwing the active exception,
# try/except/else where the except branch short-circuits else.
_ledger: list[int] = []


class MyErr(Exception):
    def __init__(self, msg):
        super().__init__(msg)
        self.msg = msg


# raise with str message — str(e) recovers it
try:
    raise ValueError("boom")
    _ledger.append(0)
except ValueError as e:
    assert str(e) == "boom"; _ledger.append(1)

# custom Exception subclass: super().__init__(msg) makes str(e) work,
# and the subclass body can keep an extra attribute on the instance
try:
    raise MyErr("custom")
    _ledger.append(0)
except MyErr as e:
    assert e.msg == "custom"; _ledger.append(1)
    assert str(e) == "custom"; _ledger.append(1)

# raise X from Y sets __cause__ to the original exception object
try:
    try:
        raise ValueError("inner")
    except ValueError as e:
        raise RuntimeError("outer") from e
except RuntimeError as e:
    assert str(e) == "outer"; _ledger.append(1)
    assert e.__cause__ is not None; _ledger.append(1)
    assert str(e.__cause__) == "inner"; _ledger.append(1)

# Implicit __context__: raise inside an except branch records the
# active exception as the new exception's __context__
try:
    try:
        raise ValueError("first")
    except ValueError:
        raise RuntimeError("second")
except RuntimeError as e:
    assert str(e) == "second"; _ledger.append(1)
    assert e.__context__ is not None; _ledger.append(1)
    assert str(e.__context__) == "first"; _ledger.append(1)

# `assert False, "msg"` produces AssertionError whose str is the msg
try:
    assert False, "assertmsg"
    _ledger.append(0)
except AssertionError as e:
    assert str(e) == "assertmsg"; _ledger.append(1)

# Bare `raise` inside an except re-throws the active exception
def rethrow():
    try:
        raise ValueError("re")
    except ValueError:
        raise

try:
    rethrow()
    _ledger.append(0)
except ValueError as e:
    assert str(e) == "re"; _ledger.append(1)

# try/except/else: when except fires, else is skipped
def caught():
    try:
        raise ValueError("x")
    except ValueError:
        return "caught"
    else:
        return "else"

assert caught() == "caught"; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: lang_raise_exception_chain {sum(_ledger)} asserts")
