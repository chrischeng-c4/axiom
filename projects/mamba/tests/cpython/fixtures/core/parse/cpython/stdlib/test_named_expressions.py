# RUN: parse
# Extracted from CPython 3.12 Lib/test/test_named_expressions.py — syntax constructs only.


# --- Basic walrus operator ---

(x := 10)
(y := "hello")
(z := [1, 2, 3])
(w := None)
(b := True)


# --- Walrus in if conditions ---

if (n := 10) > 5:
    pass

if (s := "hello") == "hello":
    pass

data = [1, 2, 3]
if (length := len(data)) > 0:
    result = length * 2

value = ""
if (stripped := value.strip()):
    result = stripped
else:
    result = "default"

if (x := 10) > 5 and (y := 20) > 15:
    total = x + y

if (a := 1) or (b := 2):
    pass


# --- Walrus in while conditions ---

items = [1, 2, 3, 4, 5]
index = 0

def read_next():
    return "data"

# Pattern: while (val := expr) check
while (item := items[index] if index < len(items) else None) is not None:
    index += 1
    if index >= len(items):
        break

# Pattern: while True with walrus
chunk_list = ["a", "b", ""]
idx = 0
while (chunk := chunk_list[idx] if idx < len(chunk_list) else ""):
    idx += 1

# While with walrus and break
counter = 10
while (val := counter) > 0:
    counter -= 1
    if val == 5:
        break


# --- Walrus in list comprehensions ---

data = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
result = [y for x in data if (y := x * 2) > 8]

result = [clean for raw in ["  hi  ", "", "  bye  "] if (clean := raw.strip())]

result = [(y := x ** 2, y + 1) for x in range(5)]

result = [y for x in range(10) if (y := x ** 2) > 20 if y < 80]


# --- Walrus in set comprehensions ---

seen = set()
data = [1, 2, 2, 3, 3, 3, 4]
unique_doubled = {y for x in data if (y := x * 2) not in seen}


# --- Walrus in dict comprehensions ---

items = ["hello", "world", "hi", "hey"]
result = {word: length for word in items if (length := len(word)) > 2}


# --- Walrus in generator expressions ---

data = range(20)
gen = (y for x in data if (y := x ** 2) > 50)

total = sum(y for x in range(10) if (y := x * 3) > 10)


# --- Walrus in conditional expressions (ternary) ---

x = 10
result = (y := x * 2) if x > 5 else (y := x * 3)

values = [1, 2, 3]
msg = f"long: {n}" if (n := len(values)) > 2 else f"short: {n}"


# --- Walrus in function arguments ---

def process(val):
    return val * 2

result = process(x := 42)
result = process((x := 42))


# --- Walrus in assertions ---

data = [1, 2, 3]
assert (n := len(data)) == 3, f"expected 3, got {n}"
assert (val := data[0]) > 0


# --- Walrus in return / yield ---

def compute(data):
    return (result := sum(data)), result * 2

def gen_with_walrus(items):
    for item in items:
        if (processed := item * 2) > 5:
            yield processed


# --- Walrus with string operations ---

text = "  Hello, World!  "
if (clean := text.strip()):
    upper = clean.upper()

line = "key=value"
if (pos := line.find("=")) >= 0:
    key = line[:pos]
    val = line[pos + 1:]


# --- Walrus in nested conditions ---

x = 15
if (a := x // 10) > 0:
    if (b := x % 10) > 0:
        result = a * 10 + b


# --- Walrus in exception handling ---

def might_fail():
    return 42

try:
    if (result := might_fail()) is not None:
        pass
except Exception as e:
    pass


# --- Walrus in f-strings ---

x = 42
msg = f"value is {(n := x * 2)} and double is {n * 2}"


# --- Walrus in match statements ---

command = "move 10 20"
match command.split():
    case ["move", x, y] if (dist := int(x) + int(y)) > 0:
        result = dist
    case _:
        result = 0


# --- Walrus with lambda ---

f = lambda: (x := 10, x + 1)
g = lambda data: (n := len(data), n > 0)


# --- Walrus in complex boolean expressions ---

data = {"key": "value"}
if (val := data.get("key")) is not None and len(val) > 0:
    result = val.upper()

items = [1, 2, 3, 4, 5]
if (first := items[0]) > 0 and (last := items[-1]) > first:
    span = last - first

# Short-circuit evaluation with walrus
x = None
result = (y := x) is not None and y > 0

items = []
result = (n := len(items)) > 0 and items[n - 1] > 0


# --- Walrus in class body ---

class Config:
    defaults = {"timeout": 30, "retries": 3}

    timeout = (t := defaults["timeout"])
    max_timeout = t * 10


# --- Walrus in with statement ---

class DummyCtx:
    def __enter__(self):
        return 42
    def __exit__(self, *args):
        pass

with DummyCtx() as val:
    if (doubled := val * 2) > 50:
        result = doubled


# --- Multiple walrus in single expression ---

coords = (3, 4)
dist = ((x := coords[0]) ** 2 + (y := coords[1]) ** 2) ** 0.5

data = [10, 20, 30]
stats = ((mn := min(data)), (mx := max(data)), mx - mn)


# --- Walrus in nested comprehensions ---

matrix = [[1, 2, 3], [4, 5, 6], [7, 8, 9]]
flat_big = [
    val
    for row in matrix
    if (row_sum := sum(row)) > 10
    for val in row
    if val > 3
]


# --- Walrus with type checks ---

mixed = [1, "two", 3, "four", 5]
ints_only = [
    n for item in mixed
    if isinstance((n := item), int)
]
