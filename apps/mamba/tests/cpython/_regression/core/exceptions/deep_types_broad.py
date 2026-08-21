# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# exceptions deep broad

# KeyError
try:
    d = {"a": 1}
    x = d["missing"]
except KeyError as e:
    print("key:", e)

# ValueError from int()
try:
    int("not a number")
except ValueError as e:
    print("val")

# ZeroDivisionError int
try:
    x = 10 / 0
except ZeroDivisionError as e:
    print("zdi")

# ZeroDivisionError //
try:
    x = 10 // 0
except ZeroDivisionError as e:
    print("zdfloor")

# catch base Exception
try:
    raise RuntimeError("runtime")
except Exception as e:
    print("exc base:", e)

# else clause
def f_else(should_raise):
    try:
        if should_raise:
            raise ValueError("bad")
        x = 1
    except ValueError:
        return "err"
    else:
        return "ok"

print(f_else(True))
print(f_else(False))

# reraise
def rr():
    try:
        try:
            raise ValueError("inner")
        except ValueError:
            raise
    except ValueError as e:
        return str(e)

print(rr())

# multiple except with different types
def handle(x):
    try:
        if x == 1:
            raise ValueError("v")
        elif x == 2:
            raise KeyError("k")
        elif x == 3:
            raise RuntimeError("r")
    except ValueError:
        return "val"
    except KeyError:
        return "key"
    except Exception:
        return "other"

print(handle(1))
print(handle(2))
print(handle(3))

# exception with type check via class
class DBError(Exception):
    pass

class ConnErr(DBError):
    pass

class QueryErr(DBError):
    pass

try:
    raise ConnErr("conn")
except DBError as e:
    print("db base:", e)

# more-specific-first wins
try:
    raise QueryErr("q")
except ConnErr:
    print("wrong")
except QueryErr as e:
    print("right:", e)
except DBError:
    print("wrong-base")