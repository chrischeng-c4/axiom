# Operational AssertionPass seed for ternary conditional-expression
# patterns beyond lang_if_elif_ternary. Surface: the `a if cond else b`
# ternary in various positions — directly returned from a function
# (including a chained sign() / grade() pattern with multiple cascaded
# `else if … else …` layers); assigned to a local; inside a list
# comprehension as both a filter (`if x > 2` at the end) and as a
# transform (`("yes" if x > 2 else "no") for x in …`) and combined;
# as a function default-argument expression; as a dict-value expression
# (per-key True/False branch); inside an f-string brace expression.
# All return-value paths use strings to dodge int-return identity.
_ledger: list[int] = []

# Chained ternary in a function body
def sign(n: int) -> str:
    return "pos" if n > 0 else "neg" if n < 0 else "zero"
assert sign(5) == "pos"; _ledger.append(1)
assert sign(-3) == "neg"; _ledger.append(1)
assert sign(0) == "zero"; _ledger.append(1)

# Ternary in assignment
v = 10
label = "big" if v > 5 else "small"
assert label == "big"; _ledger.append(1)
v = 3
label = "big" if v > 5 else "small"
assert label == "small"; _ledger.append(1)
v = 5
label = "big" if v > 5 else "small"
assert label == "small"; _ledger.append(1)

# List comprehension — filter form (trailing if guards which elements
# are kept; no transform on the bound value)
assert [x for x in range(5) if x > 2] == [3, 4]; _ledger.append(1)
assert [x for x in range(10) if x % 2 == 0] == [0, 2, 4, 6, 8]; _ledger.append(1)

# List comprehension — transform form (ternary in the expression
# position; every element produces a value, the ternary picks which)
assert [("yes" if x > 2 else "no") for x in range(5)] == ["no", "no", "no", "yes", "yes"]; _ledger.append(1)
# Combined filter + transform
assert [x * 2 for x in range(5) if x > 1] == [4, 6, 8]; _ledger.append(1)

# Three-layer chained ternary — grade classifier
def grade(s: int) -> str:
    return "A" if s >= 90 else "B" if s >= 80 else "C" if s >= 70 else "F"
assert grade(95) == "A"; _ledger.append(1)
assert grade(85) == "B"; _ledger.append(1)
assert grade(75) == "C"; _ledger.append(1)
assert grade(50) == "F"; _ledger.append(1)
assert grade(90) == "A"; _ledger.append(1)
assert grade(80) == "B"; _ledger.append(1)
assert grade(70) == "C"; _ledger.append(1)

# Ternary as default-argument expression — evaluated at function
# definition time, not per-call
def greet(name: str = "World" if True else "Default") -> str:
    return "Hello " + name
assert greet() == "Hello World"; _ledger.append(1)
assert greet("alice") == "Hello alice"; _ledger.append(1)

# Ternary as dict value — per-key True/False branch
d = {"a": 1 if True else 2, "b": 1 if False else 2}
assert d == {"a": 1, "b": 2}; _ledger.append(1)

# Ternary inside an f-string brace expression
n = 5
assert f"{'big' if n > 3 else 'small'}" == "big"; _ledger.append(1)

# Ternary returning differently-typed strings based on conditional
def classify(x: int) -> str:
    return "even" if x % 2 == 0 else "odd"
assert classify(0) == "even"; _ledger.append(1)
assert classify(1) == "odd"; _ledger.append(1)
assert classify(4) == "even"; _ledger.append(1)
assert classify(7) == "odd"; _ledger.append(1)

# Ternary in a list comprehension that distinguishes negative / zero / positive
labels = [("neg" if x < 0 else "zero" if x == 0 else "pos") for x in [-1, 0, 1, -5, 3]]
assert labels == ["neg", "zero", "pos", "neg", "pos"]; _ledger.append(1)

# Ternary with `not` — toggles between two strings
flag = True
val = "on" if not (not flag) else "off"
assert val == "on"; _ledger.append(1)
flag = False
val = "on" if not (not flag) else "off"
assert val == "off"; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: lang_conditional_expression {sum(_ledger)} asserts")
