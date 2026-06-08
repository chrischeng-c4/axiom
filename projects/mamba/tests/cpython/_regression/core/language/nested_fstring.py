# Language conformance: nested f-string evaluation (R4).
# Tests recursive f-string parsing per PEP 701.

# TC-4.1: Simple nested f-string with literal
print(f"{f'{42}'}")

# TC-4.2: Nested f-string with variable
x = 5
print(f"{f'{x}'}")

# TC-4.4: Nested f-string with expression
print(f"{f'{1 + 2}'}")

# TC-4.5: Three-level nested f-string
print(f"a{f"b{f"c"}"}")

# TC-4.6: Non-nested f-string (regression guard)
x = 10
print(f"val={x}")
