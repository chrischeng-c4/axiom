# enumerate() with start= parameter

# Default start=0
xs = ["a", "b", "c"]
for i, v in enumerate(xs):
    print(i, v)

# start=1
for i, v in enumerate(xs, start=1):
    print(i, v)

# start=10
for i, v in enumerate(xs, 10):
    print(i, v)

# start=-3
for i, v in enumerate(xs, -3):
    print(i, v)

# enumerate on iterator (not just list)
def gen():
    yield "x"
    yield "y"
    yield "z"

for i, v in enumerate(gen(), 100):
    print(i, v)

# list(enumerate(...))
print(list(enumerate(["p", "q"], 5)))
