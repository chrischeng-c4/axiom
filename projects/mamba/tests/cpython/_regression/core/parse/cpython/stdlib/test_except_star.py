# RUN: parse
# Extracted from CPython 3.12 Lib/test/test_except_star.py — except* syntax constructs only.
import sys


# --- Basic except* with single type ---

try:
    raise ValueError(1)
except* ValueError as e:
    pass


# --- except* with tuple of exception types ---

try:
    raise ExceptionGroup("eg", [TypeError(1), ValueError(2)])
except* (TypeError, ValueError) as e:
    pass


# --- except* unnamed (no 'as' binding) ---

try:
    raise ValueError(1)
except* ValueError:
    pass


# --- Multiple except* clauses ---

try:
    raise ExceptionGroup("eg", [TypeError(1), ValueError(2), OSError(3)])
except* TypeError as e:
    pass
except* ValueError as e:
    pass
except* OSError as e:
    pass


# --- except* with else clause ---

try:
    pass
except* ValueError:
    pass
else:
    x = 42


# --- except* with finally clause ---

try:
    raise ExceptionGroup("eg", [ValueError(1)])
except* ValueError:
    pass
finally:
    cleanup = True


# --- Nested except* ---

try:
    raise ExceptionGroup("outer", [TypeError(1)])
except* TypeError:
    try:
        raise ExceptionGroup("inner", [ValueError(2)])
    except* ValueError:
        pass


# --- except* in a loop ---

for _ in range(2):
    try:
        raise ExceptionGroup("loop", [TypeError(1)])
    except* TypeError:
        pass


# --- except* with reraise ---

try:
    try:
        raise ExceptionGroup("eg", [TypeError(1), ValueError(2)])
    except* TypeError as e:
        raise
    except* ValueError as e:
        pass
except ExceptionGroup as e:
    exc = e


# --- except* with raise new exception ---

try:
    try:
        raise ExceptionGroup("eg", [ValueError(1), OSError(2)])
    except* OSError as e:
        raise TypeError(3)
except ExceptionGroup as e:
    exc = e


# --- except* with raise from ---

try:
    try:
        raise ExceptionGroup("eg", [ValueError(1), OSError(2)])
    except* OSError as e:
        raise TypeError(3) from e
except ExceptionGroup as e:
    exc = e


# --- ExceptionGroup creation and nesting ---

eg1 = ExceptionGroup("test1", [ValueError("V"), TypeError("T")])
eg2 = ExceptionGroup("test2", [ValueError("V1"), ValueError("V2")])
eg3 = ExceptionGroup(
    "nested",
    [ValueError("V1"),
     OSError("OS1"),
     ExceptionGroup("inner", [OSError("OS2"), ValueError("V2"), TypeError("T")])])


# --- except* matching with supertype ---

try:
    raise ExceptionGroup("st", [BlockingIOError("io"), TypeError("T")])
except* OSError as e:
    pass
except* TypeError as e:
    pass


# --- except* with plain (non-group) exception ---

try:
    raise ValueError(42)
except* ValueError as e:
    pass


# --- ExceptionGroup subclass with derive ---

class CustomEG(ExceptionGroup):
    def __new__(cls, message, excs, code=0):
        obj = super().__new__(cls, message, excs)
        obj.code = code
        return obj

    def derive(self, excs):
        return CustomEG(self.message, excs, self.code)


# --- Falsy ExceptionGroup subclass ---

class FalsyEG(ExceptionGroup):
    def __bool__(self):
        return False

    def derive(self, excs):
        return FalsyEG(self.message, excs)


# --- except* in except handler ---

try:
    raise ExceptionGroup("eg", [TypeError(1), ValueError(2)])
except Exception:
    try:
        raise ExceptionGroup("inner", [OSError(3)])
    except* OSError:
        pass


# --- except* in except* handler ---

try:
    raise ExceptionGroup("eg", [TypeError(1), ValueError(2)])
except* Exception:
    try:
        raise ExceptionGroup("inner", [OSError(3)])
    except* OSError:
        pass


# --- sys.exception() in except* ---

try:
    raise ValueError(42)
except* ValueError as e:
    sys_exc = sys.exception()


# --- Weird leaf exceptions (unhashable, bad __eq__) ---

class UnhashableExc(ValueError):
    __hash__ = None

class AlwaysEqualExc(ValueError):
    def __eq__(self, other):
        return True

class NeverEqualExc(ValueError):
    def __eq__(self, other):
        return False

class BrokenEqualExc(ValueError):
    def __eq__(self, other):
        raise RuntimeError()


# --- Weird ExceptionGroup subclasses ---

class UnhashableEG(ExceptionGroup):
    __hash__ = None
    def derive(self, excs):
        return type(self)(self.message, excs)

class AlwaysEqualEG(ExceptionGroup):
    def __eq__(self, other):
        return True
    def derive(self, excs):
        return type(self)(self.message, excs)

class NeverEqualEG(ExceptionGroup):
    def __eq__(self, other):
        return False
    def derive(self, excs):
        return type(self)(self.message, excs)

class BrokenEqualEG(ExceptionGroup):
    def __eq__(self, other):
        raise RuntimeError()
    def derive(self, excs):
        return type(self)(self.message, excs)


# --- Complex nesting: except* with try/except inside ---

try:
    try:
        raise TypeError(2)
    except TypeError as te:
        raise ExceptionGroup("nested", [te]) from None
except ExceptionGroup as nested:
    try:
        raise ValueError(1)
    except ValueError as ve:
        raise ExceptionGroup("eg", [ve, nested])


# --- Break/continue valid inside except* (in nested loop) ---

try:
    raise ValueError(42)
except* Exception as e:
    count = 0
    for i in range(5):
        if i == 0:
            continue
        if i == 4:
            break
        count += 1


# --- Function return valid inside except* (in nested function) ---

try:
    raise ValueError(42)
except* Exception as e:
    def f(x):
        return 2 * x
    r = f(3)


# --- except* cleanup: sys.exception restored ---

try:
    try:
        raise ValueError(42)
    except:
        try:
            raise TypeError(int)
        except* Exception:
            pass
        1 / 0
except Exception as e:
    exc = e
