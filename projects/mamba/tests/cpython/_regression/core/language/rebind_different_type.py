# Regression: Python allows rebinding a name to a value of a different
# type. Mamba's type checker rejected this as "type mismatch in
# assignment" for any reassignment that widened the symbol's known
# type. Identifier targets now widen to Any when the incoming value
# doesn't fit the committed type.

a = 5
a = "hello"
print(a)

a = [1, 2, 3]
print(a)

a = 3.14
print(a)

# Rebind through a function call
def make():
    return "world"

b = 42
b = make()
print(b)

# Structural targets (attr / index) must still be type-checked so
# collections maintain coherence. Use a plain dict/list — the current
# target_ty vs value_ty check is lenient enough for these.
d = {"k": 1}
d["k"] = 2
print(d)

lst = [10, 20]
lst[0] = 99
print(lst)
