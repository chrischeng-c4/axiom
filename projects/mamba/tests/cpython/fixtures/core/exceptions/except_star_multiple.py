# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# except* with multiple handler types

try:
    raise ExceptionGroup("multi", [
        ValueError("bad value"),
        TypeError("bad type"),
        KeyError("bad key"),
    ])
except* ValueError as eg:
    print(f"ValueError group: {len(eg.exceptions)} exceptions")
except* TypeError as eg:
    print(f"TypeError group: {len(eg.exceptions)} exceptions")
except* KeyError as eg:
    print(f"KeyError group: {len(eg.exceptions)} exceptions")