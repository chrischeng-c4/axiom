# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# exception messages / handlers broad

# basic try/except catch type
try:
    raise ValueError("hello")
except ValueError as e:
    print("caught", e)

# catch base class
try:
    raise ValueError("specific")
except Exception as e:
    print("caught broadly", e)

# multiple except clauses
def categorize(x):
    try:
        if x == 0:
            raise ZeroDivisionError("zero")
        elif x < 0:
            raise ValueError("negative")
        else:
            raise TypeError("positive")
    except ZeroDivisionError as e:
        return "zd:" + str(e)
    except ValueError as e:
        return "ve:" + str(e)
    except TypeError as e:
        return "te:" + str(e)

print(categorize(0))
print(categorize(-1))
print(categorize(1))

# try with no catch of matching
def no_match(x):
    try:
        try:
            raise ValueError("inner")
        except TypeError:
            return "caught-by-type"
    except ValueError as e:
        return "caught-outer:" + str(e)

print(no_match(1))

# nested try
def nested():
    try:
        try:
            raise ValueError("inner")
        except ValueError as e:
            return "caught-inner:" + str(e)
    except Exception as e:
        return "caught-outer:" + str(e)

print(nested())

# exception propagation
def raises():
    raise ValueError("propagated")

def caller():
    try:
        raises()
        return "no-exc"
    except ValueError as e:
        return "caught:" + str(e)

print(caller())

# raise with no message
try:
    raise ValueError()
except ValueError:
    print("caught no-msg")

# raise RuntimeError, IndexError
try:
    raise RuntimeError("rt")
except RuntimeError as e:
    print("RT:", e)

try:
    raise IndexError("out")
except IndexError as e:
    print("IE:", e)

# try-except-else (ran when no exception)
def te():
    try:
        x = 10
    except ValueError:
        return "err"
    else:
        return "ok:" + str(x)

print(te())

# try-except-else (not ran on exception)
def te2():
    try:
        raise ValueError("v")
    except ValueError:
        return "caught"
    else:
        return "ok"

print(te2())

# finally without return
try:
    print("try1")
finally:
    print("finally1")

try:
    raise ValueError("v2")
except ValueError:
    print("caught2")
finally:
    print("finally2")