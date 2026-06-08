# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Regression: caught exceptions must expose __context__ and __cause__
# via attribute access. Previously both returned None even when a
# chained exception was attached at raise time.

# Implicit chaining via `raise X` inside an except handler
try:
    try:
        raise ValueError("inner")
    except ValueError:
        raise RuntimeError("outer")
except RuntimeError as e:
    print("type:", type(e).__name__)
    print("msg:", e)
    print("ctx_type:", type(e.__context__).__name__ if e.__context__ else "None")

# Explicit chaining via `raise X from Y`
try:
    try:
        raise ValueError("first")
    except ValueError as exc:
        raise RuntimeError("second") from exc
except RuntimeError as e:
    print("cause_type:", type(e.__cause__).__name__ if e.__cause__ else "None")

# `raise X from None` suppresses context
try:
    try:
        raise ValueError("hidden")
    except ValueError:
        raise RuntimeError("plain") from None
except RuntimeError as e:
    print("cause_none:", e.__cause__ is None)
    print("suppressed:", e.__suppress_context__)

# No chain — defaults
try:
    raise ValueError("stand-alone")
except ValueError as e:
    print("no_ctx:", e.__context__ is None)
    print("no_cause:", e.__cause__ is None)