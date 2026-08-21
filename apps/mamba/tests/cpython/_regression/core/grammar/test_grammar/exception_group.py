# RUN: parse
# Parser gracefully accepts except* (ExceptionGroup) syntax

try:
    raise ExceptionGroup("group", [ValueError("a"), TypeError("b")])
except* ValueError as eg:
    print("caught ValueError group:", eg)
except* TypeError as eg:
    print("caught TypeError group:", eg)
