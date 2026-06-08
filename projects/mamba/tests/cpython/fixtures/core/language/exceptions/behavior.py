"""Behavior contract for language exceptions.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

# Rule 1: try/except catches matching exception
_caught = False
try:
    raise ValueError("oops")
except ValueError:
    _caught = True
assert _caught

# Rule 2: except doesn't catch unrelated exception
_caught2 = False
try:
    try:
        raise TypeError("wrong type")
    except ValueError:
        _caught2 = True
except TypeError:
    pass
assert not _caught2, "except caught wrong type"

# Rule 3: except catches base class exceptions
_caught3 = False
try:
    raise FileNotFoundError("file")  # subclass of OSError
except OSError:
    _caught3 = True
assert _caught3, "base class not caught"

# Rule 4: except tuple catches any of the listed exceptions
for exc_type in (ValueError, TypeError, KeyError):
    _caught4 = False
    try:
        raise exc_type("x")
    except (ValueError, TypeError, KeyError):
        _caught4 = True
    assert _caught4, f"{exc_type.__name__} not caught by tuple"

# Rule 5: finally always runs (even with return)
_log = []
def _with_finally() -> int:
    try:
        return 42
    finally:
        _log.append("finally")
assert _with_finally() == 42
assert _log == ["finally"], f"finally log = {_log!r}"

# Rule 6: else runs only when no exception raised
_ran = []
try:
    pass
except Exception:
    _ran.append("except")
else:
    _ran.append("else")
assert _ran == ["else"], f"ran = {_ran!r}"

# Rule 7: raise re-raises in except block
_msgs = []
try:
    try:
        raise ValueError("original")
    except ValueError as e:
        _msgs.append(str(e))
        raise
except ValueError as e2:
    _msgs.append(str(e2))
assert _msgs == ["original", "original"], f"re-raise msgs = {_msgs!r}"

# Rule 8: raise from sets __cause__
try:
    try:
        raise ValueError("root")
    except ValueError as root_e:
        raise RuntimeError("wrapped") from root_e
except RuntimeError as e:
    assert e.__cause__ is not None, "__cause__ not set"
    assert isinstance(e.__cause__, ValueError), "__cause__ wrong type"
    assert str(e.__cause__) == "root", f"__cause__ msg = {str(e.__cause__)!r}"

# Rule 9: raise from None suppresses __context__
try:
    try:
        raise ValueError("ignored")
    except ValueError:
        raise RuntimeError("clean") from None
except RuntimeError as e:
    assert e.__cause__ is None, "__cause__ should be None"
    assert e.__suppress_context__ == True, "__suppress_context__ should be True"

# Rule 10: custom exception with custom attribute
class _AppError(Exception):
    def __init__(self, code: int, msg: str):
        super().__init__(msg)
        self.code = code
try:
    raise _AppError(404, "not found")
except _AppError as e:
    assert e.code == 404, f"code = {e.code!r}"
    assert str(e) == "not found", f"str(e) = {str(e)!r}"

# Rule 11: exception variable deleted after except block (PEP 3110)
try:
    raise ValueError("x")
except ValueError as exc:
    _exc_ref = exc
_raised = False
try:
    _ = exc  # type: ignore[possibly-undefined]
except NameError:
    _raised = True
assert _raised, "exception variable not deleted after except block"

print("behavior OK")
