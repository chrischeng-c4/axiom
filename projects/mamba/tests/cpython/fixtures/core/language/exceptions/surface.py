"""Surface contract for language exceptions.

# type-regime: monomorphic

Probes: try/except, try/finally, try/except/else, raise, raise from,
built-in exception hierarchy, custom exception classes, exception args.
CPython 3.12 is the oracle.
"""

# try / except catches exception
try:
    raise ValueError("test")
except ValueError:
    caught = True
assert caught == True, "except not executed"

# except as e captures exception
try:
    raise RuntimeError("msg")
except RuntimeError as e:
    assert str(e) == "msg", f"str(e) = {str(e)!r}"
    assert isinstance(e, RuntimeError)
    assert isinstance(e, Exception)
    assert isinstance(e, BaseException)

# try / finally always runs
finally_ran = False
try:
    x = 1 / 1
finally:
    finally_ran = True
assert finally_ran == True, "finally not executed"

# try / except / else — else runs when no exception
else_ran = False
try:
    pass
except Exception:
    pass
else:
    else_ran = True
assert else_ran == True, "else not executed"

# Custom exception subclass
class _MyError(ValueError):
    pass
_raised = False
try:
    raise _MyError("custom")
except ValueError:
    _raised = True
assert _raised, "custom exception not caught as ValueError"

# Exception args
try:
    raise OSError(42, "not found")
except OSError as e:
    assert e.args == (42, "not found"), f"e.args = {e.args!r}"

print("surface OK")
