# match statement broad

# literal match
def classify(x):
    match x:
        case 0:
            return "zero"
        case 1 | 2 | 3:
            return "small"
        case 10 | 20 | 30:
            return "tens"
        case _:
            return "other"

print(classify(0))
print(classify(2))
print(classify(20))
print(classify(99))

# sequence pattern
def describe_list(xs):
    match xs:
        case []:
            return "empty"
        case [x]:
            return f"single: {x}"
        case [x, y]:
            return f"pair: {x}, {y}"
        case [x, *rest]:
            return f"head {x}, rest {rest}"

print(describe_list([]))
print(describe_list([1]))
print(describe_list([1, 2]))
print(describe_list([1, 2, 3, 4, 5]))

# dict pattern
def check_config(cfg):
    match cfg:
        case {"type": "user", "name": name}:
            return f"user: {name}"
        case {"type": "admin", "name": name}:
            return f"admin: {name}"
        case {"type": t}:
            return f"unknown type: {t}"
        case _:
            return "no type"

print(check_config({"type": "user", "name": "Alice"}))
print(check_config({"type": "admin", "name": "Bob"}))
print(check_config({"type": "guest"}))
print(check_config({}))

# guard
def sign_match(n):
    match n:
        case x if x > 0:
            return "pos"
        case x if x < 0:
            return "neg"
        case _:
            return "zero"

print(sign_match(5))
print(sign_match(-5))
print(sign_match(0))

# value binding
def first_item(xs):
    match xs:
        case [first, *_]:
            return first
        case _:
            return None

print(first_item([1, 2, 3]))
print(first_item([]))
print(first_item([99]))
