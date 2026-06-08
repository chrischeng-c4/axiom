# RUN: parse

# Nested f-strings (PEP 701 - Python 3.12)
s = f"{"nested"}"
s = f"{f"inner {1 + 2}"}"
s = f"outer {f"inner {f"deep"}"}"

# Multi-line f-string expressions (PEP 701)
result = f"""
result = {
    1 + 2
}
"""
