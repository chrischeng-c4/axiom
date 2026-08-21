# closure / nested function patterns broad

# basic nested
def outer():
    x = 10
    def inner():
        return x
    return inner()
print(outer())

# nested 3 levels read-only
def triple():
    x = 1
    def level2():
        y = 2
        def level3():
            return x + y
        return level3()
    return level2()
print(triple())

# closure with list (append mutates outer)
def make_collector():
    items = []
    def add(x):
        items.append(x)
        return len(items)
    return add

coll = make_collector()
print(coll(1))
print(coll(2))
print(coll(3))

# closure over outer variable (read-only)
def outer2():
    msg = "hi"
    def inner():
        return msg + "!"
    return inner()
print(outer2())

# closure reads multi outer vars
def mixed():
    a = 10
    b = 20
    c = 30
    def sum_them():
        return a + b + c
    return sum_them()
print(mixed())
