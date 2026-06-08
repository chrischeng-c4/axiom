# conditional / control flow broad

# if/elif/else chain
def classify(n):
    if n < 0:
        return "neg"
    elif n == 0:
        return "zero"
    elif n < 10:
        return "small"
    elif n < 100:
        return "med"
    else:
        return "big"

print(classify(-5))
print(classify(0))
print(classify(5))
print(classify(50))
print(classify(500))

# nested
def grade(score, bonus):
    if score >= 90:
        if bonus:
            return "A+"
        return "A"
    elif score >= 70:
        return "B"
    else:
        return "F"

print(grade(95, True))
print(grade(95, False))
print(grade(80, True))
print(grade(50, False))

# ternary
def sign(n):
    return "pos" if n > 0 else "neg" if n < 0 else "zero"

print(sign(5))
print(sign(-5))
print(sign(0))

# chained condition
def check_range(x):
    return "valid" if 0 <= x <= 100 else "invalid"

print(check_range(50))
print(check_range(-1))
print(check_range(101))

# for with break/continue
def first_even(xs):
    for x in xs:
        if x % 2 != 0:
            continue
        return x
    return None

print(first_even([1, 3, 5, 4, 7]))
print(first_even([1, 3, 5]))

# nested for
total = 0
for i in range(5):
    for j in range(5):
        if (i + j) % 2 == 0:
            total += i + j
print(total)

# while with break
def countdown(n):
    result = []
    while True:
        if n <= 0:
            break
        result.append(n)
        n -= 1
    return result

print(countdown(5))
print(countdown(0))

# return None explicit
def returns_none():
    return None

print(returns_none())
