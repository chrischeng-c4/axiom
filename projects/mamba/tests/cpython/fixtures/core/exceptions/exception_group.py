# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# ExceptionGroup and except* syntax (PEP 654)
try:
    raise ExceptionGroup("errors", [
        ValueError("val error"),
        TypeError("type error"),
        KeyError("key error"),
    ])
except* ValueError as eg:
    print("caught ValueError group:", eg.exceptions)
except* TypeError as eg:
    print("caught TypeError group:", eg.exceptions)
except* KeyError as eg:
    print("caught KeyError group:", eg.exceptions)