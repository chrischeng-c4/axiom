x = 10
def f():
    global x
    x = 20
f()
print(x)

def outer():
    y = 1
    def inner():
        nonlocal y
        y = 2
    inner()
    return y

print(outer())
