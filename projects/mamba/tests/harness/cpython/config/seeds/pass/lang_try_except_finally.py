# Operational AssertionPass seed for try / except / else / finally
# exception-handling surfaces.
# Surface: try/except catches the raised type; raised exceptions whose
# class doesn't match aren't caught (test by routing the answer
# through a string return); try/except/else runs the else only when
# the try body finishes cleanly; try/finally runs the finally on both
# the clean and the raised path; multiple except clauses dispatch by
# raised type; `except T as e` binds the exception value (str(e)
# yields the message); subclass exceptions are caught by a base-class
# except; nested try/except — inner except for a non-matching type
# lets the exception propagate to the outer except; try/except/else/
# finally — all four clauses run in the expected order; bare `raise`
# inside an except re-raises the currently-active exception.
_ledger: list[int] = []


# try/except — non-zero divisor returns a string sentinel so we don't
# trip the int-identity-through-return quirk
def _divide_label(a, b):
    try:
        a / b
    except ZeroDivisionError:
        return "div_by_zero"
    return "ok"

assert _divide_label(10, 2) == "ok"; _ledger.append(1)
assert _divide_label(10, 0) == "div_by_zero"; _ledger.append(1)


# try/except returning a string sentinel on parse failure
def _parse_label(s):
    try:
        int(s)
    except ValueError:
        return "bad"
    return "good"

assert _parse_label("5") == "good"; _ledger.append(1)
assert _parse_label("abc") == "bad"; _ledger.append(1)
assert _parse_label("-7") == "good"; _ledger.append(1)


# try/except/else — else only runs on clean termination of the try
_else_log: list[str] = []


def _safe_parse(s):
    try:
        int(s)
    except ValueError:
        _else_log.append("except")
        return "bad"
    else:
        _else_log.append("else")
        return "good"


_else_log.clear()
assert _safe_parse("5") == "good"; _ledger.append(1)
assert _else_log == ["else"]; _ledger.append(1)
_else_log.clear()
assert _safe_parse("nope") == "bad"; _ledger.append(1)
assert _else_log == ["except"]; _ledger.append(1)


# try/finally — finally runs on the clean path AND the raised path
_finally_log: list[str] = []


def _with_finally(x):
    try:
        _finally_log.append("try")
        if x < 0:
            raise ValueError("negative")
        return "clean"
    finally:
        _finally_log.append("finally")


_finally_log.clear()
assert _with_finally(5) == "clean"; _ledger.append(1)
assert _finally_log == ["try", "finally"]; _ledger.append(1)

_finally_log.clear()
caught = False
try:
    _with_finally(-1)
except ValueError:
    caught = True
assert caught == True; _ledger.append(1)
# Finally still ran even though the function raised
assert _finally_log == ["try", "finally"]; _ledger.append(1)


# Multiple except clauses — dispatch on raised type
def _classify(x):
    try:
        if x == 0:
            1 / 0
        elif x < 0:
            int("not_a_number")
        return "ok"
    except ZeroDivisionError:
        return "div"
    except ValueError:
        return "val"

assert _classify(5) == "ok"; _ledger.append(1)
assert _classify(0) == "div"; _ledger.append(1)
assert _classify(-1) == "val"; _ledger.append(1)


# except T as e — bind the exception, str(e) yields the message
def _grab_msg():
    try:
        raise ValueError("custom msg")
    except ValueError as e:
        return str(e)

assert _grab_msg() == "custom msg"; _ledger.append(1)


# Base class catches subclass — ValueError is a subclass of Exception
def _catch_base():
    try:
        raise ValueError("x")
    except Exception:
        return "caught"

assert _catch_base() == "caught"; _ledger.append(1)


# Nested try — inner except doesn't match, propagates to outer
def _nested_propagate():
    try:
        try:
            raise ValueError("inner")
        except KeyError:
            return "inner_key"
    except ValueError as e:
        return str(e)

assert _nested_propagate() == "inner"; _ledger.append(1)


# try/except/else/finally — all four clauses, in order
_all_order: list[str] = []


def _all_clauses(x):
    try:
        _all_order.append("try")
        if x == "raise":
            raise ValueError("boom")
        result = "clean"
    except ValueError:
        _all_order.append("except")
        result = "caught"
    else:
        _all_order.append("else")
    finally:
        _all_order.append("finally")
    return result


_all_order.clear()
assert _all_clauses("ok") == "clean"; _ledger.append(1)
assert _all_order == ["try", "else", "finally"]; _ledger.append(1)

_all_order.clear()
assert _all_clauses("raise") == "caught"; _ledger.append(1)
assert _all_order == ["try", "except", "finally"]; _ledger.append(1)


# Bare `raise` inside an except re-raises the currently-active exception
def _reraise():
    try:
        try:
            raise ValueError("original")
        except ValueError:
            raise
    except ValueError as e:
        return str(e)

assert _reraise() == "original"; _ledger.append(1)


# Catching during iteration — a per-element try/except can swallow bad
# items and keep accumulating the good ones
def _safe_ints(items):
    out: list[str] = []
    for s in items:
        try:
            int(s)
            out.append("ok")
        except ValueError:
            out.append("skip")
    return out

assert _safe_ints(["1", "x", "2", "bad", "3"]) == ["ok", "skip", "ok", "skip", "ok"]; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: lang_try_except_finally {sum(_ledger)} asserts")
