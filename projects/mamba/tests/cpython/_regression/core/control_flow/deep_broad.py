# control flow deeper broad

# while-else: else runs when loop completes without break
def wh1():
    i = 0
    while i < 3:
        print("w", i)
        i += 1
    else:
        print("w-else")

wh1()

# while-else: else skipped when break
def wh2():
    i = 0
    while i < 10:
        if i == 2:
            break
        print("w2", i)
        i += 1
    else:
        print("w2-else (NOT)")
    print("after w2")

wh2()

# for-else: completion
def fe1():
    for x in [1, 2, 3]:
        print("f", x)
    else:
        print("f-else")

fe1()

# for-else: break skips else
def fe2():
    for x in [1, 2, 3, 4]:
        if x == 3:
            break
        print("f2", x)
    else:
        print("f2-else (NOT)")
    print("after f2")

fe2()

# nested break only exits inner
def nest_break():
    for i in range(3):
        for j in range(3):
            if j == 1:
                break
            print(i, j)

nest_break()

# continue in loop
def c1():
    for i in range(10):
        if i % 2 == 0:
            continue
        print(i)

c1()

# continue in while
def c2():
    i = 0
    while i < 10:
        i += 1
        if i % 3 == 0:
            continue
        print(i)

c2()

# elif chain
def grade_num(score):
    if score >= 90:
        return 4
    elif score >= 80:
        return 3
    elif score >= 70:
        return 2
    elif score >= 60:
        return 1
    else:
        return 0

print(grade_num(95))
print(grade_num(85))
print(grade_num(75))
print(grade_num(65))
print(grade_num(55))

# pass statement
def do_nothing():
    pass

do_nothing()
print("post-pass")

# pass in class
class Empty:
    pass

e = Empty()
print(type(e).__name__)

# pass in loop branch
def filter_positive(nums):
    result = []
    for n in nums:
        if n > 0:
            result.append(n)
        else:
            pass
    return result

print(filter_positive([-2, -1, 0, 1, 2, 3]))
