def add(a, b, c): return a + b + c
args = [1, 2, 3]
print(add(*args))

def identity(x): return x
words = ["world"]
print(identity(*words))

def identity2(x): return x
tup = (42,)
print(identity2(*tup))
