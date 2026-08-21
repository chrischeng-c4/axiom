# Operational AssertionPass seed for while / break / continue
# control-flow surfaces.
# Surface: basic while loop terminates when the head condition becomes
# False; `break` exits the innermost loop immediately; `continue`
# skips the rest of the current iteration and re-tests the head;
# `while-else` runs after a clean termination; `while-else` is skipped
# when the loop exits via break; nested while loops share the same
# semantics — an inner break only exits the inner loop; `while False`
# never enters the body; `while True` requires an explicit break.
_ledger: list[int] = []

# Basic while — increment until the head condition becomes False
i = 0
while i < 5:
    i = i + 1
assert i == 5; _ledger.append(1)

# while False — body never runs
ran = False
while False:
    ran = True
assert ran == False; _ledger.append(1)

# while True with break — required to exit
j = 0
while True:
    if j >= 3:
        break
    j = j + 1
assert j == 3; _ledger.append(1)

# `break` exits the innermost loop
k = 0
while k < 10:
    if k == 4:
        break
    k = k + 1
assert k == 4; _ledger.append(1)

# `continue` skips the rest of the iteration but keeps looping
acc: list[int] = []
m = 0
while m < 5:
    m = m + 1
    if m == 3:
        continue
    acc.append(m)
assert acc == [1, 2, 4, 5]; _ledger.append(1)

# while-else — else runs on clean termination
n = 0
else_ran = False
while n < 3:
    n = n + 1
else:
    else_ran = True
assert n == 3; _ledger.append(1)
assert else_ran == True; _ledger.append(1)

# while-else — else is skipped when loop exits via break
p = 0
else_skip = False
while p < 5:
    if p == 2:
        break
    p = p + 1
else:
    else_skip = True
assert p == 2; _ledger.append(1)
assert else_skip == False; _ledger.append(1)

# Nested while — inner break only exits inner loop
outer = 0
total = 0
while outer < 3:
    inner = 0
    while inner < 5:
        if inner == 2:
            break
        inner = inner + 1
        total = total + 1
    outer = outer + 1
assert outer == 3; _ledger.append(1)
assert total == 6; _ledger.append(1)

# while with a list — keep popping until empty
stack = [1, 2, 3, 4, 5]
collected: list[int] = []
while len(stack) > 0:
    collected.append(stack.pop())
assert collected == [5, 4, 3, 2, 1]; _ledger.append(1)
assert stack == []; _ledger.append(1)

# while-else after popping a list cleanly
s2 = [10, 20]
out2: list[int] = []
finished = False
while len(s2) > 0:
    out2.append(s2.pop())
else:
    finished = True
assert out2 == [20, 10]; _ledger.append(1)
assert finished == True; _ledger.append(1)

# continue inside a nested while — only affects the inner loop's
# current iteration
ox = 0
collected2: list[int] = []
while ox < 3:
    ix = 0
    while ix < 4:
        ix = ix + 1
        if ix == 2:
            continue
        collected2.append(ix)
    ox = ox + 1
# Each outer iteration appends [1, 3, 4] — three iterations total
assert collected2 == [1, 3, 4, 1, 3, 4, 1, 3, 4]; _ledger.append(1)

# break inside a deeply nested condition still only breaks the
# nearest enclosing loop
found = False
target = 7
needle = -1
y = 0
while y < 10:
    if y == target:
        needle = y
        found = True
        break
    y = y + 1
assert found == True; _ledger.append(1)
assert needle == 7; _ledger.append(1)
assert y == 7; _ledger.append(1)

# while-else with a condition that never becomes True
# — body never runs, else still runs (clean termination of a 0-iteration loop)
zero_iter_else = False
zz = 10
while zz < 0:
    zz = zz + 1
else:
    zero_iter_else = True
assert zero_iter_else == True; _ledger.append(1)
assert zz == 10; _ledger.append(1)

# while with a bool flag — flip the flag in the body to terminate
running = True
flips = 0
while running:
    flips = flips + 1
    if flips >= 5:
        running = False
assert running == False; _ledger.append(1)
assert flips == 5; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: lang_while_break_continue {sum(_ledger)} asserts")
