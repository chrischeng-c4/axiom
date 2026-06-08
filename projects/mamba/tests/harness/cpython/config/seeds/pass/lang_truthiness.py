# Operational AssertionPass seed for Python's truthiness rules.
# Surface: every standard value has a well-defined boolean value;
# 0 / 0.0 / None / "" / [] / {} / () / set() are falsy; non-zero
# numbers, non-empty strings, lists, dicts, tuples, sets are truthy;
# `if` evaluates the condition's truthiness, not strict bool;
# `and` returns the first falsy operand or the last operand; `or`
# returns the first truthy operand or the last operand; both
# operators short-circuit so the right-hand side is not evaluated
# when the result is already determined.
_ledger: list[int] = []

# Falsy values
assert bool(0) == False; _ledger.append(1)
assert bool(0.0) == False; _ledger.append(1)
assert bool(None) == False; _ledger.append(1)
assert bool("") == False; _ledger.append(1)
assert bool([]) == False; _ledger.append(1)
assert bool({}) == False; _ledger.append(1)
assert bool(()) == False; _ledger.append(1)
assert bool(set()) == False; _ledger.append(1)

# Truthy values
assert bool(1) == True; _ledger.append(1)
assert bool(-1) == True; _ledger.append(1)
assert bool(0.1) == True; _ledger.append(1)
assert bool("x") == True; _ledger.append(1)
assert bool([0]) == True; _ledger.append(1)
assert bool({"a": 1}) == True; _ledger.append(1)
assert bool((0,)) == True; _ledger.append(1)
assert bool({1}) == True; _ledger.append(1)

# `if` consults truthiness, not strict equality with True
branch = "miss"
if 0:
    branch = "zero"
else:
    branch = "else"
assert branch == "else"; _ledger.append(1)

if []:
    branch = "list"
else:
    branch = "empty-list"
assert branch == "empty-list"; _ledger.append(1)

if "x":
    branch = "str"
else:
    branch = "missed"
assert branch == "str"; _ledger.append(1)

if 1:
    branch = "one"
else:
    branch = "missed"
assert branch == "one"; _ledger.append(1)

# `and` returns the first falsy operand, or the last operand if all
# are truthy. The result is the operand value, not coerced to bool.
assert (1 and 2) == 2; _ledger.append(1)
assert (0 and "x") == 0; _ledger.append(1)
assert ("" and "y") == ""; _ledger.append(1)
# Chain of and's: first falsy wins
assert (1 and 2 and 3) == 3; _ledger.append(1)
assert (1 and 0 and 3) == 0; _ledger.append(1)

# `or` returns the first truthy operand, or the last operand if all
# are falsy.
assert (0 or "x") == "x"; _ledger.append(1)
assert ("" or "y") == "y"; _ledger.append(1)
assert (1 or 2) == 1; _ledger.append(1)
assert (0 or 0) == 0; _ledger.append(1)
# Chain of or's: first truthy wins
assert (0 or "" or "z") == "z"; _ledger.append(1)
assert (0 or 5 or 3) == 5; _ledger.append(1)

# Short-circuit: `and` does NOT evaluate the right operand when the
# left is falsy. If it did, the division would raise.
poison_calls = [0]
def _poison():
    poison_calls[0] += 1
    return 1 / 0

assert (False and _poison()) == False; _ledger.append(1)
assert poison_calls[0] == 0; _ledger.append(1)

# Short-circuit: `or` does NOT evaluate the right operand when the
# left is truthy.
assert (True or _poison()) == True; _ledger.append(1)
assert poison_calls[0] == 0; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: lang_truthiness {sum(_ledger)} asserts")
