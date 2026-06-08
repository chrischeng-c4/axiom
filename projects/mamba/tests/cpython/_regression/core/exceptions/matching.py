# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Exception matching via except
# Basic catch
try:
    raise ValueError("bad value")
except ValueError as e:
    print("caught ValueError")

# Subclass catching
try:
    raise KeyError("missing")
except LookupError:
    print("caught LookupError")

# Tuple of exceptions — use two handlers instead
try:
    raise TypeError("wrong type")
except ValueError:
    print("caught ValueError")
except TypeError:
    print("caught TypeError")

# Unmatched falls through
try:
    try:
        raise ValueError("inner")
    except TypeError:
        print("should not reach")
except ValueError:
    print("caught in outer")

# Bare except
try:
    raise RuntimeError("oops")
except:
    print("caught with bare except")