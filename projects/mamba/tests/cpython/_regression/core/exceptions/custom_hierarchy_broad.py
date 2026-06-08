# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# custom exceptions + raise from broad

# user-defined exception subclass
class MyError(Exception):
    pass

try:
    raise MyError("boom")
except MyError as e:
    print("caught:", e)

# subclass hierarchy
class HTTPError(Exception):
    pass

class NotFound(HTTPError):
    pass

class ServerError(HTTPError):
    pass

try:
    raise NotFound("404")
except HTTPError as e:
    print("http:", e)

try:
    raise ServerError("500")
except HTTPError as e:
    print("http:", e)

# catch sibling must match
try:
    try:
        raise NotFound("x")
    except ServerError:
        print("wrong")
except NotFound:
    print("right: NotFound")

# raise Exception(arg)
try:
    raise ValueError("bad value")
except ValueError as e:
    print("ve:", e)

# multiple except clauses — first match wins
def pick(n):
    try:
        if n == 1:
            raise ValueError("v")
        elif n == 2:
            raise TypeError("t")
        else:
            raise RuntimeError("r")
    except ValueError:
        return "V"
    except TypeError:
        return "T"
    except Exception:
        return "E"

print(pick(1))
print(pick(2))
print(pick(3))

# except as binds
try:
    raise RuntimeError("oops")
except RuntimeError as err:
    print(type(err).__name__)
    print(str(err))

# catch base catches derived
try:
    raise NotFound("deep")
except Exception as e:
    print("base caught:", e)

# nested try — inner propagates to outer
def nested():
    try:
        try:
            raise ValueError("deep")
        except TypeError:
            return "inner-type"
    except ValueError:
        return "outer-value"

print(nested())