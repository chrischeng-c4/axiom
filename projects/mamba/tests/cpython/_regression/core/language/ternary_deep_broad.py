# ternary expression / conditional broad

# basic
print("yes" if True else "no")
print("yes" if False else "no")

# with condition
x = 5
print("big" if x > 3 else "small")
print("big" if x > 100 else "small")

# numeric
print(10 if True else 20)
print(10 if False else 20)

# nested
y = 5
print("pos" if y > 0 else "neg" if y < 0 else "zero")

# nested with zero
z = 0
print("pos" if z > 0 else "neg" if z < 0 else "zero")

# nested with neg
w = -3
print("pos" if w > 0 else "neg" if w < 0 else "zero")

# inside list comp
print([x if x > 0 else -x for x in [-3, -1, 0, 1, 3]])
print([x * 2 if x % 2 == 0 else x + 1 for x in range(6)])

# inside function return
def classify(n):
    return "pos" if n > 0 else "neg" if n < 0 else "zero"

print(classify(5))
print(classify(-5))
print(classify(0))

# as arg to fn
def take(v):
    return v

print(take("a" if True else "b"))

# chained
a = 1
b = 2
c = 3
result = a if a > b else (b if b > c else c)
print(result)

# ternary in f-string
x = 10
print(f"{'even' if x % 2 == 0 else 'odd'}")
print(f"{'big' if x > 5 else 'small'}")

# ternary with string concat
name = "alice"
print("hi, " + (name if True else "anonymous"))
print("hi, " + ("someone" if True else name))

# ternary in loop
results = []
for i in range(5):
    results.append("even" if i % 2 == 0 else "odd")
print(results)

# ternary returning fn
def add(a, b):
    return a + b

def sub(a, b):
    return a - b

op = add if True else sub
print(op(10, 5))
op = add if False else sub
print(op(10, 5))

# ternary with mixed types
val = 42 if True else "default"
print(val)
val2 = 42 if False else "default"
print(val2)
