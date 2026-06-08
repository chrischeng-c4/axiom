# Language conformance: nested f-strings (PEP 701, P2-R1).
# Tests recursive f-string parsing, same-quote reuse, format specs with nested braces.

# TC-1.1: Basic nested f-string
s = f"result: {f"inner {1 + 2}"}"
print(s)

# TC-1.2: 3-level nested f-string
s = f"a {f"b {f"c"}"}"
print(s)

# TC-1.3: Same-quote reuse in f-string expression (PEP 701)
s = f"{'hello'}"
print(s)

# TC-1.4: Format spec (static alignment)
s = f"{'hi':>10}"
print(s)

# TC-1.5: Lambda in f-string expression
s = f"{(lambda x: x + 1)(5)}"
print(s)

# TC-1.6: Nested f-string with arithmetic
x = 5
y = 3
s = f"sum is {f"{x + y}"}"
print(s)
