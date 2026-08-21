# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Exception args attribute and str() conformance (#755)

# Built-in exception: single arg
try:
    raise ValueError("msg")
except ValueError as e:
    print("single:", e.args)

# Built-in exception: multiple args
try:
    raise ValueError("a", "b")
except ValueError as e:
    print("multi:", e.args)
    print("first:", e.args[0])
    print("second:", e.args[1])

# Built-in exception: no args
try:
    raise ValueError()
except ValueError as e:
    print("empty:", e.args)

# str() on exception
try:
    raise ValueError("hello")
except ValueError as e:
    print("str:", str(e))

# print() on exception
try:
    raise ValueError("world")
except ValueError as e:
    print(e)

# Custom exception subclass args
class MyError(ValueError):
    pass

try:
    raise MyError("x", "y")
except MyError as e:
    print("custom:", e.args)