# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# exception types and messages

# ValueError
try:
    raise ValueError("bad input")
except ValueError as e:
    print("VE:", e)
    print(type(e).__name__)

# KeyError
try:
    raise KeyError("missing")
except KeyError as e:
    print("KE:", type(e).__name__)

# TypeError
try:
    raise TypeError("wrong type")
except TypeError as e:
    print("TE:", e)
    print(type(e).__name__)

# IndexError
try:
    raise IndexError("out of range")
except IndexError as e:
    print("IE:", e)
    print(type(e).__name__)

# RuntimeError
try:
    raise RuntimeError("oops")
except RuntimeError as e:
    print("RE:", e)

# custom exception class
class MyError(Exception):
    pass

try:
    raise MyError("custom")
except MyError as e:
    print("My:", e)
    print(type(e).__name__)
    print(isinstance(e, Exception))

# exception with args
class DetailedError(Exception):
    def __init__(self, code, msg):
        super().__init__(msg)
        self.code = code

try:
    raise DetailedError(404, "not found")
except DetailedError as e:
    print(e.code)

# hierarchy
try:
    raise ValueError("v")
except (ValueError, TypeError) as e:
    print("caught:", type(e).__name__)

# no message
try:
    raise Exception()
except Exception as e:
    print(type(e).__name__)