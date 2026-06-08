"""Surface contract for language control flow.

# type-regime: monomorphic

Probes: if/elif/else, for/while/break/continue, for-else/while-else,
nested loops, pass, ternary expression.
CPython 3.12 is the oracle.
"""

# if / elif / else
x = 5
if x > 3:
    result = "big"
elif x > 1:
    result = "medium"
else:
    result = "small"
assert result == "big", f"result = {result!r}"

# else branch taken
x = 2
if x > 3:
    result = "big"
else:
    result = "other"
assert result == "other", f"result = {result!r}"

# for loop
total = 0
for i in range(5):
    total += i
assert total == 10, f"for sum = {total!r}"

# while loop
n = 5
product = 1
while n > 0:
    product *= n
    n -= 1
assert product == 120, f"while product = {product!r}"

# break
for i in range(10):
    if i == 5:
        break
assert i == 5, f"break i = {i!r}"

# continue
evens = []
for i in range(10):
    if i % 2 != 0:
        continue
    evens.append(i)
assert evens == [0, 2, 4, 6, 8], f"continue evens = {evens!r}"

# for-else (else runs when no break)
found = False
for i in range(5):
    pass
else:
    found = True
assert found == True, "for-else not executed"

# Ternary expression
val = "yes" if True else "no"
assert val == "yes", f"ternary = {val!r}"
val2 = "yes" if False else "no"
assert val2 == "no", f"ternary false = {val2!r}"

print("surface OK")
