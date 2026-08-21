# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# raise+catch by type
try:
    raise ValueError("bad")
except ValueError as e:
    print("caught:", e)

# multi-except
def handle(exc):
    try:
        raise exc
    except ValueError:
        return "VE"
    except KeyError:
        return "KE"
    except TypeError:
        return "TE"

print(handle(ValueError("x")))
print(handle(KeyError("y")))
print(handle(TypeError("z")))

# base class catch
try:
    raise ValueError("v")
except Exception as e:
    print("base caught:", type(e).__name__)

# finally on success
def f():
    try:
        return 1
    finally:
        print("finally ran")

print(f())

# nested try
def nested():
    try:
        try:
            raise ValueError("inner")
        except KeyError:
            return "inner-key"
    except ValueError:
        return "outer-value"

print(nested())

# reraise
def reraise():
    try:
        raise ValueError("x")
    except ValueError:
        raise

try:
    reraise()
except ValueError as e:
    print("reraised:", e)

# exception chaining
def chain():
    try:
        raise ValueError("orig")
    except ValueError as v:
        raise RuntimeError("wrapped") from v

try:
    chain()
except RuntimeError as e:
    print("chained:", e)

# multiple types in tuple
def tuple_except(exc):
    try:
        raise exc
    except (ValueError, KeyError):
        return "ok"
    except Exception:
        return "other"

print(tuple_except(ValueError("x")))
print(tuple_except(KeyError("y")))
print(tuple_except(TypeError("z")))