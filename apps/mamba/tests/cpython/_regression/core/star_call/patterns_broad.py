# star call patterns broad

def add3(a, b, c):
    return a + b + c

# *args unpack
t = (1, 2, 3)
print(add3(*t))

lst = [10, 20, 30]
print(add3(*lst))

# chained calls with unpack
def f(x, y):
    return x * y

args = (3, 4)
print(f(*args))
print(f(*args) + f(*args))

# *unpack with larger tuples
def sum5(a, b, c, d, e):
    return a + b + c + d + e

print(sum5(*[1, 2, 3, 4, 5]))
print(sum5(*(10, 20, 30, 40, 50)))

# unpack a tuple of heterogeneous values
def show(a, b, c):
    return str(a) + ":" + str(b) + ":" + str(c)

print(show(*[1, "x", True]))
