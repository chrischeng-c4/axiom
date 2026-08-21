# Language conformance: semicolon statement separator (R1).
# Tests `;` as separator between simple statements on the same line.

# TC-1.1: Two assignments separated by semicolons
a = 1; b = 2; print(a); print(b)

# TC-1.2: Print and assignment separated by semicolons
print(1); x = 2; print(x)

# TC-1.3: Trailing semicolon tolerated
x = 1; print(x);

# TC-1.4: Three statements on one line
a = 1; b = 2; c = a + b; print(c)
