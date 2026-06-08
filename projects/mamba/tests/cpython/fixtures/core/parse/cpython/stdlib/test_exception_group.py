# RUN: parse
# Extracted from CPython 3.12 Lib/test/test_exception_group.py — syntax constructs only.

# --- ExceptionGroup construction ---
eg = ExceptionGroup("errors", [ValueError("v"), TypeError("t")])

# --- nested ExceptionGroup ---
inner = ExceptionGroup("inner", [ValueError("a"), KeyError("b")])
outer = ExceptionGroup("outer", [inner, TypeError("c")])

# --- BaseExceptionGroup ---
beg = BaseExceptionGroup("base", [KeyboardInterrupt(), SystemExit(1)])

# --- except* basic ---
try:
    raise ExceptionGroup("test", [ValueError("v1"), TypeError("t1")])
except* ValueError as eg:
    pass
except* TypeError as eg:
    pass

# --- except* with tuple ---
try:
    raise ExceptionGroup("test", [ValueError("v"), KeyError("k")])
except* (ValueError, KeyError) as eg:
    pass

# --- except* multiple handlers ---
try:
    raise ExceptionGroup("multi", [
        ValueError("v"),
        TypeError("t"),
        KeyError("k"),
    ])
except* ValueError as eg:
    pass
except* TypeError as eg:
    pass
except* KeyError as eg:
    pass

# --- except* with else ---
try:
    pass
except* ValueError:
    pass
else:
    x = 1

# --- except* with finally ---
try:
    raise ExceptionGroup("test", [ValueError("v")])
except* ValueError:
    pass
finally:
    cleanup = True

# --- except* with else and finally ---
try:
    pass
except* ValueError:
    pass
else:
    y = 2
finally:
    z = 3

# --- nested try/except* ---
try:
    try:
        raise ExceptionGroup("inner", [ValueError("v")])
    except* ValueError:
        raise ExceptionGroup("outer", [TypeError("t")])
except* TypeError:
    pass

# --- ExceptionGroup in function ---
def raise_group():
    raise ExceptionGroup("func", [
        ValueError("a"),
        ValueError("b"),
        TypeError("c"),
    ])

# --- ExceptionGroup with custom exceptions ---
class CustomError(Exception):
    pass

class AnotherError(Exception):
    pass

try:
    raise ExceptionGroup("custom", [
        CustomError("custom1"),
        AnotherError("another1"),
    ])
except* CustomError:
    pass
except* AnotherError:
    pass

# --- except* re-raise ---
try:
    try:
        raise ExceptionGroup("test", [ValueError("v")])
    except* ValueError:
        raise
except* ValueError:
    pass

# --- ExceptionGroup attributes ---
eg = ExceptionGroup("test", [ValueError("a"), TypeError("b")])
_ = eg.message
_ = eg.exceptions

# --- ExceptionGroup subclass ---
class MyGroup(ExceptionGroup):
    pass

# --- derive pattern ---
eg = ExceptionGroup("test", [ValueError("a"), TypeError("b")])

# --- except* with complex body ---
try:
    raise ExceptionGroup("test", [
        ValueError("v1"),
        ValueError("v2"),
        TypeError("t1"),
    ])
except* ValueError as eg:
    for exc in eg.exceptions:
        _ = str(exc)
except* TypeError as eg:
    count = len(eg.exceptions)

# --- ExceptionGroup in class ---
class ErrorCollector:
    def __init__(self):
        self.errors = []

    def add(self, error):
        self.errors.append(error)

    def raise_all(self):
        if self.errors:
            raise ExceptionGroup("collected", self.errors)

# --- single exception in group ---
try:
    raise ExceptionGroup("single", [ValueError("only one")])
except* ValueError as eg:
    pass

# --- empty-ish patterns ---
eg = ExceptionGroup("mixed", [
    ValueError("v"),
    ExceptionGroup("nested", [TypeError("t")]),
])
