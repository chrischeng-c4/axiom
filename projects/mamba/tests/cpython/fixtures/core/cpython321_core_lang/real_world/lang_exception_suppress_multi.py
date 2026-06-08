# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_exception_suppress_multi"
# subject = "cpython321.lang_exception_suppress_multi"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/lang_exception_suppress_multi.py"
# status = "filled"
# ///
"""cpython321.lang_exception_suppress_multi: execute CPython 3.12 seed lang_exception_suppress_multi"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for `raise ... from None` cause-
# suppression and multi-class except surfaces. Surface: `raise X
# from None` clears __cause__ to None and sets __suppress_context__
# to True; the multi-class `except (T1, T2) as e:` form catches
# either exception type and binds it to `e`, with `type(e).__name__`
# revealing which one fired; the bare `raise` form inside an
# except re-raises the active exception preserving its message;
# `except <Base>:` catches subclass exceptions via isinstance;
# else-clause runs iff the try-body completed without exception;
# else-clause is skipped when an except handler fires; try/finally
# always runs finally even when the try-body raises an exception
# that is caught by the surrounding scope; wrapping one exception
# in another via `raise Outer(...) from inner` exposes the inner
# via `__cause__` and isinstance-tests through it.
# Companion to lang_raise_exception_chain (basic chain surface).
_ledger: list[int] = []

# raise ... from None — clear cause and mark suppressed
def suppress() -> None:
    try:
        1 / 0
    except ZeroDivisionError:
        raise ValueError("no cause") from None

try:
    suppress()
except ValueError as e:
    assert e.__cause__ is None; _ledger.append(1)
    assert e.__suppress_context__ == True; _ledger.append(1)
    assert str(e) == "no cause"; _ledger.append(1)

# Multi-class except — `except (T1, T2)` catches either
def check(x: int) -> str:
    try:
        if x == 0:
            raise ZeroDivisionError("z")
        if x == 1:
            raise ValueError("v")
        return "ok"
    except (ZeroDivisionError, ValueError) as e:
        return type(e).__name__

assert check(0) == "ZeroDivisionError"; _ledger.append(1)
assert check(1) == "ValueError"; _ledger.append(1)
assert check(2) == "ok"; _ledger.append(1)

# Bare `raise` inside an except re-raises with message intact
def re() -> None:
    try:
        raise RuntimeError("first")
    except RuntimeError:
        raise

try:
    re()
except RuntimeError as e:
    assert str(e) == "first"; _ledger.append(1)

# Custom hierarchy — except <Base> catches subclass
class Base(Exception):
    pass
class Derived(Base):
    pass

try:
    raise Derived("d")
except Base as e:
    assert isinstance(e, Derived); _ledger.append(1)
    assert isinstance(e, Base); _ledger.append(1)

# Wrapping via `raise Outer from inner` exposes inner as __cause__
class MyError(Exception):
    pass

try:
    try:
        raise KeyError("k")
    except KeyError as e:
        raise MyError("wrapped") from e
except MyError as e:
    assert str(e) == "wrapped"; _ledger.append(1)
    assert isinstance(e.__cause__, KeyError); _ledger.append(1)

# try/finally always runs finally — even when try-body raises
_log: list[str] = []
def fin() -> None:
    try:
        _log.append("try")
        raise ValueError("x")
    finally:
        _log.append("finally")

try:
    fin()
except ValueError:
    _log.append("caught")
assert _log == ["try", "finally", "caught"]; _ledger.append(1)

# else-clause runs iff try-body completed without exception
_log3: list[str] = []
try:
    _log3.append("try")
except ValueError:
    _log3.append("except")
else:
    _log3.append("else")
assert _log3 == ["try", "else"]; _ledger.append(1)

# else-clause skipped when except handler fires
_log4: list[str] = []
try:
    _log4.append("try")
    raise ValueError("x")
except ValueError:
    _log4.append("except")
else:
    _log4.append("else")
assert _log4 == ["try", "except"]; _ledger.append(1)

# Multiple except clauses — first matching type wins
def select(x: int) -> str:
    try:
        if x == 0:
            raise TypeError("t")
        raise ValueError("v")
    except TypeError:
        return "type"
    except ValueError:
        return "value"

assert select(0) == "type"; _ledger.append(1)
assert select(1) == "value"; _ledger.append(1)

# Implicit __context__ on raise inside except (no from)
def boom() -> None:
    try:
        1 / 0
    except ZeroDivisionError:
        raise ValueError("inner failed")

try:
    boom()
except ValueError as e:
    assert isinstance(e.__context__, ZeroDivisionError); _ledger.append(1)

# Chained except with `from e` — cause is the original
def divide(a: int, b: int) -> float:
    try:
        return a / b
    except ZeroDivisionError as e:
        raise ValueError("division failed") from e

try:
    divide(1, 0)
except ValueError as e:
    assert str(e) == "division failed"; _ledger.append(1)
    assert isinstance(e.__cause__, ZeroDivisionError); _ledger.append(1)
    assert e.__cause__ is not None; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_exception_suppress_multi {sum(_ledger)} asserts")
