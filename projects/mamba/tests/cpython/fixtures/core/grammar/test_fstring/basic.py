# RUN: parse
# CPython 3.12 test_fstring: basic f-string coverage

name = "world"
greeting = f"hello {name}"

x = 42
s = f"value is {x}"
s = f"result: {1 + 2}"
s = f"{"nested string"}"
s = f"{x!r}"
s = f"{x!s}"
s = f"{x!a}"
s = f"{x:.2f}"
s = f"{x:>10}"
s = f"{x:#010x}"

# Multiple expressions
s = f"{name!r} has value {x}"

# String concatenation with f-strings
s = "hello " f"{name}" " world"

# Empty f-string
s = f""
