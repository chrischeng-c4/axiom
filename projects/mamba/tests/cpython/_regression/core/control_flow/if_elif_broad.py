# if / elif / else patterns broad

# if/elif/else grade
def grade(score):
    if score >= 90:
        return "A"
    elif score >= 80:
        return "B"
    elif score >= 70:
        return "C"
    elif score >= 60:
        return "D"
    else:
        return "F"

print(grade(95))
print(grade(85))
print(grade(75))
print(grade(65))
print(grade(55))

# if with and/or
def classify(a, b):
    if a > 0 and b > 0:
        return "both pos"
    elif a > 0 or b > 0:
        return "one pos"
    else:
        return "neither"

print(classify(1, 1))
print(classify(1, -1))
print(classify(-1, 1))
print(classify(-1, -1))

# if with comparison chain
def in_range(x, lo, hi):
    if lo <= x <= hi:
        return True
    return False

print(in_range(5, 1, 10))
print(in_range(0, 1, 10))
print(in_range(15, 1, 10))
print(in_range(1, 1, 10))
print(in_range(10, 1, 10))

# multi-branch elif
def day_type(day):
    if day == "mon":
        return 1
    elif day == "tue":
        return 2
    elif day == "wed":
        return 3
    elif day == "fri":
        return 5
    elif day == "sun":
        return 7
    else:
        return 0

print(day_type("mon"))
print(day_type("fri"))
print(day_type("sun"))
print(day_type("xxx"))

# deeply nested with return early
def classify2(x):
    if x < 0:
        if x < -100:
            return "very neg"
        return "neg"
    if x == 0:
        return "zero"
    if x < 100:
        return "small pos"
    return "big pos"

print(classify2(-200))
print(classify2(-5))
print(classify2(0))
print(classify2(50))
print(classify2(500))
