# match statement deep broad

# literal
def check_literal(x):
    match x:
        case 1:
            return "one"
        case 2:
            return "two"
        case 3:
            return "three"
        case _:
            return "other"

print(check_literal(1))
print(check_literal(2))
print(check_literal(3))
print(check_literal(99))

# or pattern
def day_type(d):
    match d:
        case "mon" | "tue" | "wed" | "thu" | "fri":
            return "weekday"
        case "sat" | "sun":
            return "weekend"
        case _:
            return "unknown"

print(day_type("mon"))
print(day_type("sun"))
print(day_type("xxx"))

# list/tuple pattern
def analyze(seq):
    match seq:
        case []:
            return "empty"
        case [x]:
            return "single"
        case [x, y]:
            return "pair"
        case [x, y, z]:
            return "triple"
        case _:
            return "many"

print(analyze([]))
print(analyze([1]))
print(analyze([1, 2]))
print(analyze([1, 2, 3]))
print(analyze([1, 2, 3, 4, 5]))

# head/tail
def head_tail(seq):
    match seq:
        case []:
            return "empty"
        case [first, *rest]:
            return "first=" + str(first) + " rest=" + str(rest)

print(head_tail([]))
print(head_tail([1]))
print(head_tail([1, 2, 3, 4]))

# guard
def sign(x):
    match x:
        case n if n < 0:
            return "neg"
        case 0:
            return "zero"
        case n if n > 0:
            return "pos"

print(sign(-5))
print(sign(0))
print(sign(5))

# match string
def is_greeting(s):
    match s:
        case "hi" | "hello" | "hey":
            return "yes"
        case _:
            return "no"

print(is_greeting("hi"))
print(is_greeting("nope"))

