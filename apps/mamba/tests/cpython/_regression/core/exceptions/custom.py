# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Custom exception subclassing
class MyError(Exception):
    pass

class DetailedError(Exception):
    def __init__(self, msg, code):
        super().__init__(msg)
        self.code = code

# Basic custom exception
try:
    raise MyError("custom error")
except MyError as e:
    print("caught MyError:", e)

# Custom with extra attribute
try:
    raise DetailedError("bad request", 400)
except DetailedError as e:
    print("caught DetailedError:", e)
    print("code:", e.code)

# Custom exception is subclass of Exception
try:
    raise MyError("test")
except Exception as e:
    print("caught via Exception:", type(e).__name__)

# args attribute
try:
    raise MyError("a", "b", "c")
except MyError as e:
    print("args:", e.args)