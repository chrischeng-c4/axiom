# RUN: parse

x = 42
# Debug format specifier (Python 3.8+)
s = f"{x=}"
s = f"{x = }"
s = f"{x + 1 = }"
s = f"{x=!r}"
s = f"{x=:.2f}"
