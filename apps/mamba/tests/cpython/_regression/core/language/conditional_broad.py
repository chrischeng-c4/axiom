# conditional / boolean logic

# ternary expressions
print(5 if True else 10)
print(5 if False else 10)
print("pos" if 5 > 0 else "neg")
print(1 if 1 > 0 else 2 if 2 > 0 else 3)  # nested
print(10 if 10 > 20 else 20 if 20 > 30 else 30 if 30 > 40 else 40)

# short-circuit and/or
print(True and "a")
print(False and "a")
print(True or "a")
print(False or "a")
print(None or "default")
print(0 or "fallback")
print("" or "empty")
print([] or "empty")
print({} or "empty")

# chained and
print(1 and 2 and 3)
print(1 and 0 and 3)
print(0 and 1 and 2)

# chained or
print(None or 0 or "found")
print("" or None or 0)

# complex
def first_true(*args):
    for a in args:
        if a:
            return a
    return None

print(first_true(0, "", [], "hi"))
print(first_true(0, "", []))

# comparison chains
x = 5
print(0 < x < 10)
print(0 < x < 3)

# all/any with comprehension
nums = [1, 2, 3, 4, 5]
print(all(n > 0 for n in nums))
print(any(n > 10 for n in nums))

# if-elif-else
def classify(n):
    if n < 0:
        return "neg"
    elif n == 0:
        return "zero"
    elif n < 10:
        return "small"
    elif n < 100:
        return "medium"
    else:
        return "large"

print(classify(-5))
print(classify(0))
print(classify(7))
print(classify(50))
print(classify(500))
