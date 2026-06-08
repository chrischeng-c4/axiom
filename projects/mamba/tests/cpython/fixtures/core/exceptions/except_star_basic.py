# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# except* basic: catch specific exception type from group

try:
    raise ExceptionGroup("errors", [
        ValueError("v1"),
        ValueError("v2"),
        TypeError("t1"),
    ])
except* ValueError as eg:
    for e in eg.exceptions:
        print(f"caught ValueError: {e}")
except* TypeError as eg:
    for e in eg.exceptions:
        print(f"caught TypeError: {e}")