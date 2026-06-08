# misc statements broad: assert, ternary, del

# assert true (bool only)
assert True
assert 1 == 1
assert 2 > 1
print("assertions passed")

# ternary
x = "yes" if True else "no"
print(x)

y = "a" if 1 < 2 else "b"
print(y)

# ternary in comprehension
vals = [1, 2, 3, 4, 5]
signs = ["even" if v % 2 == 0 else "odd" for v in vals]
print(signs)

# nested ternary
def grade(n):
    return "A" if n >= 90 else ("B" if n >= 80 else ("C" if n >= 70 else "F"))

print(grade(95))
print(grade(85))
print(grade(75))
print(grade(50))

# ternary with function calls
def double(n):
    return n * 2

def triple(n):
    return n * 3

v = double(5) if True else triple(5)
print(v)
v2 = double(5) if False else triple(5)
print(v2)

# del list element
li = [10, 20, 30, 40]
del li[0]
print(li)
del li[-1]
print(li)

# del dict key
d = {"a": 1, "b": 2, "c": 3}
del d["a"]
print(sorted(d.items()))
