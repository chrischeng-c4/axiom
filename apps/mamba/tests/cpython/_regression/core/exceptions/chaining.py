# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Exception chaining with raise from
try:
    try:
        raise ValueError("original")
    except ValueError as e:
        raise TypeError("converted") from e
except TypeError as e:
    print("caught:", e)
    print("cause:", e.__cause__)

# Implicit chaining (__context__)
try:
    try:
        raise ValueError("first")
    except ValueError:
        raise TypeError("second")
except TypeError as e:
    print("caught:", e)
    print("context:", e.__context__)

# Suppress context with from None
try:
    try:
        raise ValueError("original")
    except ValueError:
        raise TypeError("clean") from None
except TypeError as e:
    print("caught:", e)
    print("cause:", e.__cause__)
    print("suppress:", e.__suppress_context__)