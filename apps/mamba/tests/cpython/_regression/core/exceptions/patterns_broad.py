# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# exception patterns broad

# raise and catch
try:
    raise ValueError("bad")
except ValueError as e:
    print("caught:", str(e))

# raise RuntimeError
try:
    raise RuntimeError("oops")
except RuntimeError as e:
    print("rt:", str(e))

# multiple except
def classify(e):
    try:
        raise e
    except ValueError:
        return "value"
    except RuntimeError:
        return "runtime"

print(classify(ValueError("x")))
print(classify(RuntimeError("y")))

# catch base Exception
try:
    raise TypeError("t")
except Exception as e:
    print("general:", str(e))

# reraise
def top():
    try:
        inner()
    except ValueError as e:
        print("top caught:", str(e))

def inner():
    raise ValueError("from inner")

top()

# try/finally at module level
log = []
try:
    log.append("try")
finally:
    log.append("finally")
print(log)

# try/except/else with no exception
def okfn():
    try:
        x = 1 + 1
    except Exception:
        return "err"
    else:
        return "ok"

print(okfn())

# try/except/else with exception
def badfn():
    try:
        raise ValueError()
    except ValueError:
        return "err"
    else:
        return "ok"

print(badfn())

# nested try
def nested():
    try:
        try:
            raise ValueError("inner")
        except ValueError as e:
            raise RuntimeError("outer from " + str(e))
    except RuntimeError as e:
        return str(e)

print(nested())

# catch and use exception type
def show_type(e):
    try:
        raise e
    except Exception as ex:
        return type(ex).__name__

print(show_type(ValueError("x")))
print(show_type(RuntimeError("y")))
print(show_type(TypeError("z")))

# custom exception
class MyError(Exception):
    pass

try:
    raise MyError("custom")
except MyError as e:
    print("my:", str(e))

# catch custom with Exception
try:
    raise MyError("base catch")
except Exception as e:
    print("base:", str(e))