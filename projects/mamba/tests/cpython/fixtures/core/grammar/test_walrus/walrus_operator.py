# RUN: parse
# CPython 3.12 test_walrus: assignment expressions (PEP 572)

# Basic walrus in if
if (n := 10) > 5:
    pass

# Walrus in while
data = [1, 2, 3]
while (item := data.pop()) if data else None:
    pass

# Walrus in comprehension filter
results = [y for x in range(20) if (y := x ** 2) > 50]

# Walrus in function argument
import re
text = "hello 123 world"
if (m := re.search(r"\d+", text)) is not None:
    digits = m.group()

# Nested walrus
if (a := 10) > 5 and (b := a * 2) > 15:
    pass

# Walrus in assert
assert (val := 42) == 42

# Walrus with ternary
result = x if (x := 10) > 5 else 0
