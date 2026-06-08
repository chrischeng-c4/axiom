# Match edge cases: nested patterns, tuple/string literals, None/True/False

# Nested sequence patterns
def nested_seq(v):
    match v:
        case [[a, b], [c, d]]:
            return f"two pairs: {a}/{b}, {c}/{d}"
        case [[x]]:
            return f"single nested: {x}"
        case []:
            return "empty"
        case _:
            return "other"

print(nested_seq([[1, 2], [3, 4]]))
print(nested_seq([[9]]))
print(nested_seq([]))
print(nested_seq([1, 2]))

# String literal in match
def keyword_match(v):
    match v:
        case "yes":
            return 1
        case "no":
            return 0
        case _:
            return -1

print(keyword_match("yes"))
print(keyword_match("no"))
print(keyword_match("maybe"))
