# misc edge cases

# multi-line string
s = """
hello
world
"""
print(s.strip())

# raw string
print(r"\n is literal")
print(r"path\to\file")

# string repeating
print("=" * 20)
print("-" * 0)
print("ab" * 5)

# multiline expression
total = (
    1 + 2
    + 3 + 4
    + 5
)
print(total)

# parenthesized binding
result = (lambda x: x * 2)(5)
print(result)

# function returning function
def make_adder(n):
    return lambda x: x + n

add5 = make_adder(5)
print(add5(3))
print(add5(10))

# function in dict
ops = {
    "add": lambda a, b: a + b,
    "sub": lambda a, b: a - b,
    "mul": lambda a, b: a * b,
}

print(ops["add"](2, 3))
print(ops["sub"](10, 4))
print(ops["mul"](3, 7))

# function as arg
def apply(f, x):
    return f(x)

print(apply(abs, -5))
print(apply(len, "hello"))

# nested function calls
print(len(str(12345)))
print(sum(list(range(11))))
print(sorted(set([3, 1, 4, 1, 5, 9, 2, 6])))

# multiple returns
def divmod_ish(a, b):
    return a // b, a % b

q, r = divmod_ish(17, 5)
print(q, r)
