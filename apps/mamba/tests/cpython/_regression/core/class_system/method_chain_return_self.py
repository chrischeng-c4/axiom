# Regression: method chains like `c.f().f()` where f returns self produced UAF
# because the JIT Return terminator transferred a borrowed parameter as if it
# were owned, under-retaining the shared pointer.

class Builder:
    def __init__(self):
        self.items = []

    def add(self, x):
        self.items.append(x)
        return self

    def build(self):
        return self.items


b = Builder()
print(b.add(1).add(2).add(3).build())

# Direct chain inline with print.
b2 = Builder()
print(b2.add("x").add("y").build())

# Longer chain to exercise multiple aliased returns — verify reachability,
# not identity (mamba's default __repr__ differs from CPython's <__main__.X>).
class Echo:
    label = "echo"
    def f(self):
        return self

e = Echo()
print(e.f().label)
print(e.f().f().label)
print(e.f().f().f().label)
print(e.f().f().f().f().f().label)

# Chain via local variable splits (baseline — always worked).
x = e.f()
y = x.f()
print(y.label)
