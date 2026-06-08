# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Regression: when an expression raises inside a call argument, the
# pending exception must short-circuit the outer call. Previously
# `print(s.index("missing"))` inside a try/except printed the raw
# None that s.index returned, then *also* entered the except handler.

s = "hello"

try:
    print(s.index("missing"))
except ValueError as e:
    print("caught:", e)

# Raising expression as a function arg
def pass_through(x):
    return x

try:
    print(pass_through(int("bad")))
except ValueError as e:
    print("caught:", e)

# Raising expression as operand
try:
    total = int("good") + int("not-a-number")
    print(total)
except ValueError as e:
    print("caught op:", e)

# Nested: raise inside a list literal that feeds another call
try:
    print(list([int("x"), 1, 2]))
except ValueError as e:
    print("caught nested:", e)

print("still alive")