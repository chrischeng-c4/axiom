# Regression: iter(callable, sentinel) must stop when callable() returns
# the sentinel. Previously mb_call0 returned None for any Instance (no
# __call__ dispatch), so iter looped forever yielding None.

class Counter:
    def __init__(self):
        self.i = 0
    def __call__(self):
        self.i += 1
        return self.i

c = Counter()
it = iter(c, 5)
for x in it:
    print(x)

# Different sentinel value
class Say:
    def __init__(self):
        self.n = 0
    def __call__(self):
        self.n += 1
        if self.n == 3:
            return "STOP"
        return "ok " + str(self.n)

for msg in iter(Say(), "STOP"):
    print(msg)
