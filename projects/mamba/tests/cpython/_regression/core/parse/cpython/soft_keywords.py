# RUN: parse
# Soft keywords interaction tests: match, case, type (#569)

# --- "match" and "case" as variable names (they are soft keywords) ---
match = 42
# NOTE: "case" as standalone assignment at module level not supported by parser
# case = "hello"
# NOTE: "case" as assignment target not supported even inside block
# case = "hello"
type = int

# --- match as function name ---
def match(x):
    return x

# --- case as function name ---
# NOTE: "case" as function name not supported (treated as keyword)
# def case(x):
#     return x
def case_func(x):
    return x

# --- type as function name ---
def type(x):
    return x

# --- match as parameter ---
# NOTE: "case" as parameter name not supported
# def foo(match, case, type):
#     return match + case
def foo(match, case_param, type):
    return match + case_param

# --- match as class name ---
# NOTE: "match" and "case" as class names not supported
# class match:
#     pass
# class case:
#     pass
class MatchClass:
    pass
class CaseClass:
    pass

# --- match as attribute ---
# NOTE: "case" as class attribute name not supported
class Obj:
    match = 1
    # case = 2
    case_attr = 2
    type = 3

obj = Obj()
x = obj.match
# y = obj.case
y = obj.case_attr
z = obj.type

# --- match statement using soft keywords ---
match match:
    case 42:
        pass
    case _:
        pass

# --- match on variable named "case" ---
# NOTE: "case" as match subject not supported (treated as keyword)
# match case:
#     case "hello":
#         pass

# --- nested match with soft keyword names ---
match = [1, 2, 3]
match match:
    case [1, *rest]:
        pass
    case _:
        pass

# --- type as keyword in assignment ---
type = "string"
x = type

# --- type statement (PEP 695) using "type" soft keyword ---
type Point = tuple[int, int]
type Vector = list[float]

# --- type in comprehension ---
types = [type for type in [int, str, float]]

# --- match in comprehension ---
matches = [match for match in range(10)]

# --- case in dict ---
d = {"case": 1, "match": 2, "type": 3}

# --- import with soft keyword names ---
# (these are valid as module names conceptually)
# import match  # would need actual module, skip

# --- soft keywords in f-strings ---
x = f"{match}"
# NOTE: case variable not defined; removing f-string usage
# x = f"{case}"

# --- soft keywords as decorator argument ---
# NOTE: "case" as parameter name not supported
# def decorator(match, case):
def decorator(match, case_arg):
    def wrapper(func):
        return func
    return wrapper

# NOTE: "case" as keyword argument not supported
# @decorator(match=1, case=2)
@decorator(match=1, case_arg=2)
def decorated():
    pass

# --- nested match/case complexity ---
# NOTE: match with tuple subject and "case" variable not supported
_subj = (match, case_func)
match _subj:
    case [42, s]:
        pass
    case [m, c] if m > 0:
        pass
    case _:
        pass

# --- "type" as variable in match ---
type_val = "int"
match type_val:
    case "int":
        x = int
    case "str":
        x = str
    case _:
        pass

# --- _  is also a soft keyword in match context ---
_ = 42
x = _
print(_)

# --- _ in match is wildcard, not variable ---
match 42:
    case _:
        pass

# --- match with boolean soft keyword patterns ---
match True:
    case True:
        pass
    case False:
        pass
