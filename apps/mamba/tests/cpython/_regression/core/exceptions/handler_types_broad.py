# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# exception handler type patterns broad

# ValueError
try:
    int("not a number")
except ValueError as e:
    print("caught ValueError")

# ZeroDivisionError
try:
    x = 1 / 0
except ZeroDivisionError:
    print("caught zero div")

# KeyError
try:
    d = {"a": 1}
    v = d["missing"]
except KeyError as e:
    print("caught KeyError")

# nested try
def nested():
    try:
        try:
            raise ValueError("inner")
        except ValueError:
            print("inner caught")
            raise RuntimeError("outer")
    except RuntimeError:
        print("outer caught")

nested()

# raise and re-raise
def reraise():
    try:
        raise ValueError("oops")
    except ValueError:
        print("caught, reraising")
        raise

try:
    reraise()
except ValueError:
    print("outer got it")

# Exception catches all
try:
    raise ValueError("general")
except Exception as e:
    print("caught as Exception")

# except tuple
try:
    raise ValueError("v")
except (ValueError, TypeError):
    print("caught tuple")

# specific vs general
try:
    raise RuntimeError("runtime")
except RuntimeError:
    print("caught RuntimeError specifically")