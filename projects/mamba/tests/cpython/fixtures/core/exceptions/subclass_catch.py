# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Custom exception subclass catching conformance (#755)

# Custom subclass of built-in caught by parent
class MyError(ValueError):
    pass

try:
    raise MyError("test")
except ValueError as e:
    print("caught by ValueError:", type(e).__name__)

# Deep hierarchy: B(A(ValueError)) caught by ValueError
class A(ValueError):
    pass
class B(A):
    pass

try:
    raise B("deep")
except ValueError as e:
    print("deep catch:", type(e).__name__)

# Custom subclass caught by Exception
try:
    raise MyError("exc")
except Exception as e:
    print("caught by Exception:", type(e).__name__)

# Custom exception with raise from
class OtherError(TypeError):
    pass

try:
    try:
        raise MyError("original")
    except MyError as e:
        raise OtherError("converted") from e
except OtherError as e:
    print("from cause:", str(e.__cause__))
    print("suppress:", e.__suppress_context__)

# Bare raise with custom exception
try:
    try:
        raise MyError("reraise_test")
    except MyError:
        raise
except ValueError as e:
    print("bare raise:", str(e))

# Implicit context with custom exception
try:
    try:
        raise MyError("first")
    except MyError:
        raise OtherError("second")
except OtherError as e:
    print("context:", str(e.__context__))