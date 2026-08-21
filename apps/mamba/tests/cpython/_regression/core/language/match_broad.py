def kind(x):
    match x:
        case 0:
            return "zero"
        case 1 | 2 | 3:
            return "small"
        case n if n < 0:
            return "negative"
        case _:
            return "other"

print(kind(0))
print(kind(1))
print(kind(2))
print(kind(3))
print(kind(-5))
print(kind(100))

def greeting(s):
    match s:
        case "hello":
            return "en"
        case "hola":
            return "es"
        case "bonjour":
            return "fr"
        case _:
            return "?"

print(greeting("hello"))
print(greeting("hola"))
print(greeting("bonjour"))
print(greeting("ni hao"))

def describe_point(p):
    match p:
        case (0, 0):
            return "origin"
        case (0, y):
            return f"y-axis {y}"
        case (x, 0):
            return f"x-axis {x}"
        case (x, y):
            return f"point {x},{y}"

print(describe_point((0, 0)))
print(describe_point((0, 5)))
print(describe_point((3, 0)))
print(describe_point((2, 4)))

def first_kind(lst):
    match lst:
        case []:
            return "empty"
        case [x]:
            return f"single {x}"
        case [x, y]:
            return f"pair {x},{y}"
        case [x, *rest]:
            return f"many head={x} rest={rest}"

print(first_kind([]))
print(first_kind([42]))
print(first_kind([1, 2]))
print(first_kind([1, 2, 3, 4, 5]))

def dict_match(d):
    match d:
        case {"type": "add", "x": x, "y": y}:
            return x + y
        case {"type": "mul", "x": x, "y": y}:
            return x * y
        case _:
            return None

print(dict_match({"type": "add", "x": 3, "y": 5}))
print(dict_match({"type": "mul", "x": 3, "y": 5}))
