# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""core/exception_group: BaseExceptionGroup.__new__ argument validation (3.12)."""


def expect(exc_type, fn):
    try:
        fn()
    except exc_type as e:
        return str(e)
    raise AssertionError(f"expected {exc_type.__name__}")


# The message (first arg) must be a str.
msg = expect(TypeError, lambda: ExceptionGroup(ValueError(12), [ValueError(1)]))
assert "argument 1 must be str" in msg, msg
msg = expect(TypeError, lambda: ExceptionGroup(None, [ValueError(1)]))
assert "argument 1 must be str" in msg, msg

# The exceptions (second arg) must be a sequence -- a set / None is rejected.
msg = expect(TypeError, lambda: ExceptionGroup("e", {ValueError(42)}))
assert "must be a sequence" in msg, msg
msg = expect(TypeError, lambda: ExceptionGroup("e", None))
assert "must be a sequence" in msg, msg

# The sequence must be non-empty.
msg = expect(ValueError, lambda: ExceptionGroup("e", []))
assert "must be a non-empty sequence" in msg, msg

# Every item must be an exception instance (not a class, not a str).
msg = expect(ValueError, lambda: ExceptionGroup("e", [OSError]))
assert "is not an exception" in msg, msg
msg = expect(ValueError, lambda: ExceptionGroup("e", ["not an exception"]))
assert "is not an exception" in msg, msg

# Exactly two positional arguments are required.
msg = expect(TypeError, lambda: ExceptionGroup("no errors"))
assert "takes exactly 2 arguments" in msg, msg
msg = expect(TypeError, lambda: ExceptionGroup([ValueError("no msg")]))
assert "takes exactly 2 arguments" in msg, msg
msg = expect(TypeError, lambda: ExceptionGroup("e", [ValueError(1)], [TypeError(2)]))
assert "takes exactly 2 arguments" in msg, msg

print("construction_errors OK")
