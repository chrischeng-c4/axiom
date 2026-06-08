# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# error cases with try/except (working subset)

# ZeroDivisionError
try:
    x = 10 / 0
except ZeroDivisionError:
    print("zero div")

try:
    y = 10 // 0
except ZeroDivisionError:
    print("floor zero")

# ValueError
try:
    int("not a number")
except ValueError:
    print("value")

# catch generic Exception
try:
    raise ValueError("x")
except Exception as e:
    print(type(e).__name__)
    print(str(e))

# multiple except types
def check(x):
    try:
        if x == 1:
            raise ValueError("v")
        elif x == 2:
            raise TypeError("t")
        elif x == 3:
            raise KeyError("k")
        return "ok"
    except ValueError:
        return "V"
    except TypeError:
        return "T"
    except KeyError:
        return "K"

print(check(0))
print(check(1))
print(check(2))
print(check(3))

# exception chain - raise from
try:
    try:
        raise ValueError("orig")
    except ValueError as e:
        raise RuntimeError("wrap") from e
except RuntimeError as e:
    print(str(e))