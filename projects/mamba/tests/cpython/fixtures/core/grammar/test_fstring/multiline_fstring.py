# RUN: parse

# Multi-line f-string (PEP 701 Python 3.12)
x = 10
s = f"""
The value of x is {
    x
}
and more text
"""

# Backslash in f-string expressions (PEP 701)
data = [1, 2, 3]
s = f"items: {", ".join(str(i) for i in data)}"
