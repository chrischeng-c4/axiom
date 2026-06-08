# Chained compares with variables, ne, and in-expression evaluation

# Variable bounds
x = 5
print(0 < x < 10)
print(10 < x < 20)
print(x == x == x)

# != in chain
print(1 != 2 != 3)
print(1 != 1 != 2)

# > variants
print(10 > 5 > 1)
print(10 > 5 > 6)

# Chained with function result
def f():
    return 7

print(1 < f() < 10)
print(0 < f() < 5)

# Chain used in if
v = 3
if 0 < v < 5:
    print("in range")
else:
    print("out")

# Chain in boolean expression
print((0 < v < 5) and (v < 10))

# Chain that triggers short-circuit — c should not be evaluated
log = []
def tagged(tag, val):
    log.append(tag)
    return val

# 1 < 0 is False, so no further eval
print(1 < tagged("a", 0) < tagged("b", 10))
print(log)
