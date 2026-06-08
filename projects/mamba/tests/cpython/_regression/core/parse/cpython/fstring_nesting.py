# RUN: parse
# CPython-derived: complex f-string nesting (#551)

# --- basic f-string ---
name = "world"
s = f"hello {name}"

# --- conversion flags ---
s = f"{name!r}"
s = f"{name!s}"
s = f"{name!a}"

# --- format spec ---
value = 3.14159
s = f"{value:.2f}"
s = f"{value:>10}"
s = f"{value:0>10.2f}"

# --- dynamic format spec ---
width = 10
precision = 2
s = f"{value:{width}.{precision}}"

# --- conversion with format spec ---
s = f"{'string'!s:>10}"
s = f"{'string'!r:^20}"

# --- nested f-strings ---
x = 42
s = f"{'nested: ' + f'{x}'}"
s = f"result: {f'{x + 1}'}"
s = f"deep: {f'mid: {f\"inner: {x}\"}'}"

# --- f-string with dict access ---
d = {"key": "value"}
s = f"{d['key']}"
s = f"got: {d['key']!r}"

# --- f-string with method calls ---
text = "hello"
s = f"{text.upper()}"
s = f"{text.replace('h', 'H')}"
s = f"{'hello world'.split()[0]}"

# --- f-string with conditional ---
cond = True
x = 1
y = 2
s = f"{x if cond else y}"
s = f"{'yes' if cond else 'no'}"

# --- f-string with walrus ---
s = f"{(n := 10)} and {n + 1}"

# --- multi-line f-strings ---
s = f"""
first line: {x}
second line: {y}
"""

s = f'''
triple single: {x}
quoted: {y}
'''

# --- single-quoted f-strings ---
s = f'single: {x}'

# --- double-quoted f-strings ---
s = f"double: {x}"

# --- escaped braces ---
s = f"{{literal}}"
s = f"open {{ and close }}"
s = f"{x} with {{braces}}"

# --- empty f-string ---
s = f""

# --- f-string with list comprehension ---
s = f"{[i * 2 for i in range(5)]}"

# --- f-string with lambda ---
s = f"{(lambda x: x + 1)(5)}"

# --- f-string with slice ---
items = [1, 2, 3, 4, 5]
s = f"{items[1:3]}"

# --- f-string with star expression in call ---
args = [1, 2, 3]
s = f"{max(*args)}"

# --- f-string with ternary and format spec ---
s = f"{'positive' if x > 0 else 'non-positive':>20}"

# --- f-string with complex expression ---
s = f"{sum(i ** 2 for i in range(10))}"

# --- f-string with multiple expressions ---
a = 1
b = 2
s = f"{a} + {b} = {a + b}"

# --- PEP 701 relaxed grammar (Python 3.12) ---
# Reuse of same quote type inside f-string
# NOTE: PEP 701 same-quote nesting not supported: s = f"hello {"world"}"
# NOTE: PEP 701 same-quote nesting not supported: s = f"value: {d["key"]}"

# Nested f-strings reusing quotes
# NOTE: PEP 701 same-quote nesting not supported: s = f"result: {f"inner: {x}"}"

# Multi-level nesting with same quotes
# NOTE: PEP 701 same-quote nesting not supported: s = f"a {f"b {f"c {x}"}"}"

# NOTE: These are valid in any f-string (not PEP 701 specific)
s = f"newline: {chr(10)}"
s = f"tab: {chr(9)}"

# --- f-string with complex dict ---
data = {"a": 1, "b": 2}
# NOTE: same-quote dict access in f-string not supported
# s = f"{data["a"] + data["b"]}"
s = f"{data['a'] + data['b']}"

# --- f-string with joined expressions ---
names = ["Alice", "Bob", "Charlie"]
s = f"names: {', '.join(names)}"

# --- raw f-string ---
s = rf"raw with {x} and \n literal"
s = rf"\t{x}\n"

# --- f-string with subscript ---
matrix = [[1, 2], [3, 4]]
s = f"{matrix[0][1]}"

# --- f-string with chained calls ---
s = f"{'hello world'.upper().split()[0]}"

# --- f-string with walrus and format ---
s = f"{(val := 3.14159):.2f}"
