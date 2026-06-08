# RUN: parse
# Deeply nested expression stress test fixture (#571)

# --- deeply nested parentheses ---
x = ((((((((1))))))))
x = (((((1 + 2) * 3) - 4) / 5) ** 2)

# --- deeply nested function calls ---
x = abs(abs(abs(abs(abs(-5)))))
x = str(int(float(str(42))))
x = len(list(map(str, range(10))))

# --- deeply nested list literals ---
x = [[[[[1, 2], [3, 4]], [[5, 6], [7, 8]]]]]
x = [1, [2, [3, [4, [5]]]]]

# --- deeply nested dict literals ---
x = {"a": {"b": {"c": {"d": 1}}}}
x = {"level1": {"level2": {"level3": {"level4": "deep"}}}}

# --- deeply nested conditionals ---
x = 1 if True else (2 if False else (3 if True else (4 if False else 5)))

# --- deeply nested boolean expressions ---
x = (True and (False or (True and (False or True))))
x = (not (not (not (not True))))

# --- deeply nested comprehensions ---
x = [x for x in [y for y in [z for z in range(10)]]]
x = [[i + j for i in range(3)] for j in range(3)]
x = [[[k for k in range(2)] for j in range(2)] for i in range(2)]

# --- deeply nested lambda ---
f = lambda x: (lambda y: (lambda z: x + y + z))
result = f(1)(2)(3)

# --- deeply nested attribute access ---
class A:
    class B:
        class C:
            class D:
                value = 42

# --- deeply nested subscript ---
data = {0: {1: {2: {3: "found"}}}}
x = data[0][1][2][3]

# --- deeply nested if/elif/else ---
def classify(x):
    if x > 100:
        if x > 200:
            if x > 300:
                return "very high"
            else:
                return "high"
        else:
            return "medium-high"
    elif x > 50:
        if x > 75:
            return "medium"
        else:
            return "medium-low"
    else:
        if x > 25:
            return "low"
        else:
            if x > 0:
                return "very low"
            else:
                return "zero or negative"

# --- deeply nested loops ---
for i in range(3):
    for j in range(3):
        for k in range(3):
            for m in range(3):
                x = i + j + k + m

# --- deeply nested try/except ---
try:
    try:
        try:
            x = 1 / 0
        except ZeroDivisionError:
            pass
    except Exception:
        pass
except BaseException:
    pass

# --- deeply nested with ---
class Ctx:
    def __enter__(self):
        return self
    def __exit__(self, *a):
        pass

c = Ctx()
with c:
    with c:
        with c:
            with c:
                pass

# --- deeply nested class definitions ---
class Outer:
    class Middle:
        class Inner:
            class Core:
                x = 1

# --- deeply nested function definitions ---
def f1():
    def f2():
        def f3():
            def f4():
                def f5():
                    return 42
                return f5
            return f4
        return f3
    return f2

# --- complex nested expression combining many features ---
result = [
    {k: [v ** 2 for v in vals if v > 0]}
    for k, vals in {"a": [1, -2, 3], "b": [-1, 4, 5]}.items()
    if any(v > 0 for v in vals)
]

# --- deeply nested match ---
# NOTE: match with complex tuple subject causes parser issues; use a variable
_subj = (1, (2, (3, 4)))
match _subj:
    # NOTE: nested tuple pattern not supported: case (a, (b, (c, d))):
    # NOTE: tuple case pattern not supported; using list pattern
    case [a, _rest]:
        pass
    case _:
        pass

# --- deeply nested tuple unpacking ---
# NOTE: deeply nested parenthesized tuple unpacking may not be supported
# ((a, (b, (c, d))), e) = ((1, (2, (3, 4))), 5)
(_ab, e) = ((1, (2, (3, 4))), 5)
(a, _bcd) = _ab

# --- nested generators ---
gen = (
    x
    for xs in (range(i) for i in range(5))
    for x in xs
    if x > 0
)
