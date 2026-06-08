# RUN: parse
# CPython 3.12 test_exceptions: try/except/finally

# Basic try/except
try:
    x = 1 / 0
except ZeroDivisionError:
    pass

# With as clause
try:
    raise ValueError("test")
except ValueError as e:
    msg = str(e)

# Multiple except clauses
try:
    pass
except TypeError:
    pass
except ValueError:
    pass
except (KeyError, IndexError):
    pass

# Bare except
try:
    pass
except:
    pass

# Try/except/else
try:
    result = 42
except Exception:
    result = 0
else:
    pass

# Try/finally
try:
    pass
finally:
    pass

# Full try/except/else/finally
try:
    x = 1
except Exception:
    x = 0
else:
    x = x + 1
finally:
    pass

# Nested try
try:
    try:
        raise ValueError()
    except ValueError:
        raise TypeError()
except TypeError:
    pass
