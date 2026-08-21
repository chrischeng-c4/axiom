# RUN: parse
# Extracted from CPython 3.12 Lib/test/test_exceptions.py — syntax constructs only.

# --- basic try/except ---
try:
    x = 1
except:
    pass

try:
    x = 1 / 0
except ZeroDivisionError:
    pass

try:
    x = 1 / 0
except ZeroDivisionError as e:
    pass

# --- multiple except clauses ---
try:
    x = int("abc")
except ValueError:
    pass
except TypeError:
    pass
except (KeyError, IndexError):
    pass

# --- try/except/else ---
try:
    x = 1
except Exception:
    pass
else:
    y = 2

# --- try/except/finally ---
try:
    x = 1
except Exception:
    pass
finally:
    z = 3

# --- try/except/else/finally ---
try:
    x = 1
except ValueError:
    pass
except TypeError:
    pass
else:
    y = 2
finally:
    z = 3

# --- bare raise ---
try:
    raise ValueError("test")
except ValueError:
    raise

# --- raise with cause ---
try:
    raise ValueError("test")
except ValueError as e:
    raise RuntimeError("wrapped") from e

# --- raise from None ---
try:
    raise ValueError("test")
except ValueError:
    raise RuntimeError("clean") from None

# --- nested try/except ---
try:
    try:
        x = 1 / 0
    except ZeroDivisionError:
        raise ValueError("inner")
except ValueError:
    pass

# --- exception in loop ---
for i in range(5):
    try:
        if i == 3:
            raise StopIteration
    except StopIteration:
        break

# --- exception in while ---
while True:
    try:
        raise KeyboardInterrupt
    except KeyboardInterrupt:
        break

# --- custom exception classes ---
class MyError(Exception):
    pass

class MyValueError(ValueError):
    def __init__(self, value, message="bad value"):
        self.value = value
        self.message = message
        super().__init__(self.message)

class DetailedError(Exception):
    def __init__(self, msg, code=0, details=None):
        super().__init__(msg)
        self.code = code
        self.details = details or {}

# --- exception hierarchy ---
class AppError(Exception):
    pass

class DatabaseError(AppError):
    pass

class ConnectionError(DatabaseError):
    pass

class QueryError(DatabaseError):
    pass

# --- exception chaining ---
try:
    try:
        raise ValueError("original")
    except ValueError as e:
        raise TypeError("chained") from e
except TypeError as e:
    pass

# --- assert statement ---
assert True
assert 1 == 1
assert 1 == 1, "should be equal"

# --- try with return ---
def safe_div(a, b):
    try:
        return a / b
    except ZeroDivisionError:
        return None
    finally:
        pass

# --- try with yield ---
def gen_with_except():
    try:
        yield 1
        yield 2
    except GeneratorExit:
        pass
    finally:
        pass

# --- exception in comprehension ---
results = []
for i in range(5):
    try:
        results.append(10 / i)
    except ZeroDivisionError:
        results.append(0)

# --- exception with context manager ---
class SafeContext:
    def __enter__(self):
        return self
    def __exit__(self, exc_type, exc_val, exc_tb):
        return True

with SafeContext():
    raise ValueError("suppressed")

# --- multiple exception types in one except ---
try:
    pass
except (ValueError, TypeError, KeyError) as e:
    pass

# --- BaseException hierarchy ---
try:
    raise SystemExit(0)
except SystemExit:
    pass

try:
    raise KeyboardInterrupt
except KeyboardInterrupt:
    pass

# --- exception attributes ---
try:
    raise OSError(2, "No such file", "test.txt")
except OSError as e:
    _ = e.errno
    _ = e.strerror
    _ = e.filename

# --- exception in class method ---
class Validator:
    def validate(self, value):
        if not isinstance(value, int):
            raise TypeError(f"expected int, got {type(value)}")
        if value < 0:
            raise ValueError(f"expected positive, got {value}")
        return value

# --- try/except in lambda (not possible, use function) ---
def safe_call(func, default=None):
    try:
        return func()
    except Exception:
        return default

# --- deeply nested exception handling ---
def deep_handler():
    try:
        try:
            try:
                raise ValueError("deep")
            except ValueError:
                raise TypeError("wrapped1")
        except TypeError:
            raise RuntimeError("wrapped2")
    except RuntimeError:
        pass
