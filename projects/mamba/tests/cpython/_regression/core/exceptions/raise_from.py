# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# raise X from Y — exception chaining via __cause__
# (Full __cause__/__context__ attribute access not yet implemented; test basic flow.)

try:
    try:
        raise ValueError("inner")
    except ValueError as e:
        raise RuntimeError("wrapper") from e
except RuntimeError as r:
    print(type(r).__name__)