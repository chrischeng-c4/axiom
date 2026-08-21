# RUN: parse
# PEP 701 relaxed f-string grammar tests — Python 3.12 specific (#567)

# --- basic f-strings ---
name = "world"
x = f"hello {name}"
x = f'hello {name}'

# --- expression types inside f-strings ---
x = f"{1 + 2}"
x = f"{len('hello')}"
x = f"{True if 1 > 0 else False}"
x = f"{[i for i in range(3)]}"

# --- format specs ---
pi = 3.14159
x = f"{pi:.2f}"
x = f"{pi:10.3f}"
x = f"{42:08d}"
x = f"{255:#x}"
x = f"{'hello':>20}"
x = f"{'hello':<20}"
x = f"{'hello':^20}"
x = f"{'hello':*^20}"

# --- dynamic format specs ---
width = 10
precision = 3
x = f"{pi:{width}.{precision}f}"
x = f"{'hello':{width}}"

# --- conversions ---
x = f"{name!r}"
x = f"{name!s}"
x = f"{name!a}"
x = f"{name!r:>20}"

# --- nested f-strings (PEP 701 allows reuse of quotes) ---
x = f"{'hello'}"
x = f"{f'{name}'}"
x = f"result: {f'{1 + 2}'}"

# --- dict access in f-strings ---
d = {"key": "value"}
x = f"{d['key']}"

# --- method calls ---
x = f"{'hello'.upper()}"
x = f"{[1,2,3].count(1)}"

# --- conditional expressions ---
flag = True
x = f"{'yes' if flag else 'no'}"

# --- walrus in f-strings ---
x = f"{(y := 42)}"

# --- multi-line f-strings ---
x = f"""
hello
{name}
world
"""

x = f'''
first line
{1 + 2}
last line
'''

# --- escaped braces ---
x = f"{{literal braces}}"
x = f"{{{{double escaped}}}}"
x = f"{name} has {{braces}}"

# --- empty f-string ---
x = f""
x = f''
x = f""""""
x = f''''''

# --- f-string with newlines in expression ---
x = f"{(
    1 + 2
    + 3
)}"

# --- complex expressions ---
data = [1, 2, 3]
x = f"sum={sum(data)}, len={len(data)}"
x = f"{', '.join(str(i) for i in range(5))}"
x = f"{'%.2f' % 3.14}"

# --- raw f-strings ---
x = rf"\n is {name}"
x = rf"{name}\t"

# --- bytes cannot be f-strings (just regular strings for parse test) ---
# fb"..." is not valid, skip

# --- chained f-string concatenation ---
x = f"hello " f"world"
x = f"{1}" f"{2}" f"{3}"

# --- subscript in f-strings ---
lst = [10, 20, 30]
x = f"{lst[0]}"
x = f"{lst[-1]}"
x = f"{lst[1:3]}"

# --- lambda in f-strings (needs parens) ---
x = f"{(lambda: 42)()}"

# --- ternary chains in f-strings ---
a, b, c = 1, 2, 3
x = f"{'a' if a > b else 'b' if b > c else 'c'}"
