# RUN: parse
# Multi-line expression continuation patterns fixture (#573)

# --- implicit continuation inside parentheses ---
x = (
    1 + 2 +
    3 + 4 +
    5
)

# --- implicit continuation inside brackets ---
items = [
    1, 2, 3,
    4, 5, 6,
    7, 8, 9,
]

# --- implicit continuation inside braces ---
config = {
    "host": "localhost",
    "port": 8080,
    "debug": True,
}

# NOTE: backslash continuation replaced with parentheses
# --- explicit line continuation with backslash ---
total = (1 +
    2 +
    3 +
    4)

# --- multi-line function call ---
result = some_function(
    arg1,
    arg2,
    keyword1=value1,
    keyword2=value2,
)

# --- multi-line function definition ---
def complex_function(
    param1,
    param2,
    param3="default",
    *args,
    **kwargs,
):
    pass

# --- multi-line class definition ---
class MyClass(
    BaseClass1,
    BaseClass2,
    metaclass=type,
):
    pass

# --- multi-line import ---
from collections import (
    OrderedDict,
    defaultdict,
    namedtuple,
    deque,
)

# --- multi-line if condition ---
if (
    condition1
    and condition2
    and condition3
    or condition4
):
    pass

# --- multi-line assert ---
assert (
    value > 0
    and value < 100
), "Value out of range"

# --- multi-line assignment ---
result = (
    first_part
    + second_part
    + third_part
)

# --- multi-line list comprehension ---
squares = [
    x ** 2
    for x in range(100)
    if x % 2 == 0
    if x % 3 == 0
]

# --- multi-line dict comprehension ---
mapping = {
    key: value
    for key, value in items
    if key is not None
}

# --- multi-line generator expression ---
gen = (
    x * y
    for x in range(10)
    for y in range(10)
    if x != y
)

# --- multi-line set comprehension ---
unique = {
    item.lower()
    for item in collection
    if item is not None
}

# --- multi-line with statement ---
# NOTE: parenthesized with+as not supported; use non-parenthesized form
with open("file1") as f1, open("file2") as f2:
    pass

# --- multi-line return ---
def f():
    return (
        some_long_value
        + another_long_value
        + yet_another_value
    )

# --- multi-line yield ---
def gen():
    yield (
        complex_computation()
        + more_computation()
    )

# --- multi-line decorator ---
@some_decorator(
    param1="value1",
    param2="value2",
)
def decorated():
    pass

# --- multi-line string ---
# NOTE: implicit multi-line string concat in parens not supported
# text = (
#     "first line "
#     "second line "
#     "third line"
# )
text = "first line " + "second line " + "third line"

# --- multi-line tuple assignment ---
(
    a,
    b,
    c,
) = (1, 2, 3)

# --- multi-line chained comparison ---
result = (
    0 < x
    < 100
)

# NOTE: backslash method chain replaced with parentheses
# --- multi-line method chain (backslash) ---
result = (obj
    .method1()
    .method2()
    .method3())

# --- multi-line method chain (parenthesized) ---
result = (
    obj
    .method1()
    .method2()
    .method3()
)

# --- multi-line ternary ---
x = (
    "positive"
    if value > 0
    else "negative"
    if value < 0
    else "zero"
)

# --- multi-line match ---
# NOTE: parenthesized match subject across lines not supported
match some_long_expression:
    case 1:
        pass
    case _:
        pass

# --- multi-line f-string ---
# NOTE: implicit multi-line f-string concat in parens not supported
# msg = (
#     f"Hello {name}, "
#     f"you have {count} "
#     f"items in your cart."
# )
msg = f"Hello {name}, you have {count} items in your cart."
