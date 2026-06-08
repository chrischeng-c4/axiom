# scope deep broad

# global read
x = 10
def read_global():
    return x

print(read_global())

# global write
def modify_global():
    global x
    x = 99

modify_global()
print(x)

# global int counter
count = 0
def inc():
    global count
    count += 1

inc()
inc()
inc()
print(count)

# global string
msg = "hello"
def greet(n):
    global msg
    msg = msg + n

greet(" world")
print(msg)

# local shadows global
y = "global"
def local_test():
    y = "local"
    return y

print(local_test())
print(y)

# local vs global with same name
z = 100
def uses_z():
    z = 5
    return z * 2

print(uses_z())
print(z)

# function-local is not visible outside
def make_var():
    only_local = 42
    return only_local

print(make_var())

# nested scope - read-only access
def outer_r():
    a = 10
    def inner():
        return a
    return inner()

print(outer_r())

# multiple global
a_glob = 1
b_glob = 2
def update_two():
    global a_glob, b_glob
    a_glob = 10
    b_glob = 20

update_two()
print(a_glob, b_glob)

# class scope vs instance
class Box:
    cls_val = 99
    def __init__(self):
        self.inst_val = 42
    def read_cls(self):
        return Box.cls_val

b = Box()
print(b.inst_val)
print(b.read_cls())
print(Box.cls_val)
