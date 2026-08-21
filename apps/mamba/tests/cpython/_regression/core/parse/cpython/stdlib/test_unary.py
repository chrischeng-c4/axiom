# RUN: parse
# Extracted from CPython Lib/test/test_unary.py — syntax constructs only.
"""Test compiler changes for unary ops (+, -, ~) introduced in Python 2.2"""


# Negative
-2 == 0 - 2
-0 == 0
--2 == 2
-2.0 == 0 - 2.0
-2j == 0 - 2j

# Positive
+2 == 2
+0 == 0
++2 == 2
+2.0 == 2.0
+2j == 2j

# Invert
~2 == -(2 + 1)
~0 == -1
~~2 == 2

# No overflow with large numbers
nines = "9" * 32
eval("+" + nines) == 10 ** 32 - 1
eval("-" + nines) == -(10 ** 32 - 1)
eval("~" + nines) == ~(10 ** 32 - 1)

# Negation of exponentiation (regression test for SF bug #456756)
-2 ** 3 == -8
(-2) ** 3 == -8
-2 ** 4 == -16
(-2) ** 4 == 16

# Bad types for unary ops
for op in '+', '-', '~':
    try:
        eval(op + "b'a'")
    except TypeError:
        pass
    try:
        eval(op + "'a'")
    except TypeError:
        pass

try:
    eval("~2j")
except TypeError:
    pass

try:
    eval("~2.0")
except TypeError:
    pass
