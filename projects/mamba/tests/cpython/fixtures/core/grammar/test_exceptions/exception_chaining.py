# RUN: parse
# CPython 3.12 test_exceptions: exception chaining

# Explicit chaining (raise from)
try:
    try:
        raise ValueError("original")
    except ValueError as e:
        raise TypeError("converted") from e
except TypeError:
    pass

# Suppress context (raise from None)
try:
    try:
        raise ValueError("original")
    except ValueError:
        raise TypeError("clean") from None
except TypeError:
    pass

# Implicit chaining (raise inside except)
try:
    try:
        raise ValueError("first")
    except ValueError:
        raise TypeError("second")
except TypeError:
    pass

# Re-raise
try:
    try:
        raise ValueError("test")
    except ValueError:
        raise
except ValueError:
    pass
