# RUN: parse
# CPython 3.12 test_exceptions: exception groups (PEP 654)

# except* syntax
try:
    pass
except* ValueError as eg:
    pass
except* TypeError as eg:
    pass

# Multiple except* clauses
try:
    pass
except* (ValueError, KeyError) as eg:
    pass
except* TypeError as eg:
    pass

# ExceptionGroup constructor
eg = ExceptionGroup("errors", [ValueError("v"), TypeError("t")])

# BaseExceptionGroup
beg = BaseExceptionGroup("base", [KeyboardInterrupt()])
