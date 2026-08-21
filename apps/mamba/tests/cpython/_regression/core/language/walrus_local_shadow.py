# Regression: walrus `x := expr` inside a function body must bind to a
# *local* x, not reassign an outer-scope x of the same name. Previously
# the walrus lowering ignored local_assigned_names and resolve_name
# found a module-scope symbol.

def count_to_n(n):
    i = 0
    out = []
    while (i := i + 1) <= n:
        out.append(i)
    return out

print(count_to_n(3))
print(count_to_n(5))

# Name collision with outer scope — local walrus must not touch outer
def shadow():
    x = 0
    while (x := x + 1) <= 2:
        pass
    return x

print(shadow())

# Nested walrus + conditional
def classify():
    results = []
    for v in [1, 5, 10, 15, 20]:
        if (b := v > 10):
            results.append((v, "big"))
        else:
            results.append((v, "small"))
    return results

for pair in classify():
    print(pair)
