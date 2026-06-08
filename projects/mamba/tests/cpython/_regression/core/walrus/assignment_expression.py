# mamba-xfail: walrus binding inside an `if` condition reads back as a
# denormal float (e.g. `8e-323`) instead of the int the right-hand
# function returned. Looks like the int payload is reinterpreted as a
# float bit-pattern when the walrus name is consumed inside the truthy
# branch. while / comprehension / call-argument walrus clauses already
# pass on mamba; the if-context walrus is the runtime gap that gates
# the xfail.
#
# Walrus / assignment expression (PEP 572) — #2813.
#
# Covers `:=` in `if`, `while`, comprehension, and function-call
# contexts. Asserts both the bound value (the walrus name is visible
# after the expression) and the surrounding control-flow result.
#
# Failure messages name [walrus] so runner stderr identifies the area.


# 1. Walrus in `if` condition. The name is in scope on the truthy branch
#    and remains visible after the if-statement.
def lookup(x):
    if x % 7 == 0:
        return None
    return x * 2


if (val := lookup(8)) is not None:
    print("if-walrus val=", val, "[walrus]")
else:
    print("if-walrus none [walrus FAIL]")
print("after if, val=", val, "[walrus]")

# 2. Walrus in `while` condition. The classic "read until sentinel"
#    pattern from PEP 572. We exhaust a generator and observe both the
#    last bound value and the loop body's collected values.
def stream():
    yield 1
    yield 4
    yield 9
    yield None


it = stream()
collected = []
while (item := next(it, None)) is not None:
    collected.append(item)
print("while-walrus collected=", collected, "[walrus]")

# 3. Walrus in a list comprehension. Bind a per-iteration computation
#    once and use it twice in the same clause.
data = [1, 2, 3, 4, 5, 6]
squares_over_five = [s for n in data if (s := n * n) > 5]
print("comp squares>5=", squares_over_five, "[walrus]")

# 4. Walrus in a function call argument position. PEP 572 explicitly
#    supports `:=` inside argument expressions (parenthesized).
def show(label, value):
    return label + "=" + str(value)


print("call-walrus:", show("y", (y := 10 + 5)), "y=", y, "[walrus]")
