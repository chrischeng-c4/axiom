# Regression: inside a function, `x = val` must define a local even when
# an outer-scope `x` exists. Previously the resolver + HIR treated the
# assignment as a rebind of the outer symbol, so the inner read saw the
# wrong storage slot.

x = "outer"

def f():
    x = 42  # local; outer x untouched
    print(x)

f()
print(x)

# Multiple local names that shadow
a = 1
b = 2
def g():
    a = 10
    b = 20
    print(a, b)

g()
print(a, b)

# Nested functions — inner shadows outer function's locals, not module's
def h():
    y = "h's y"
    def inner():
        y = 99
        print(y)
    inner()
    print(y)

h()
