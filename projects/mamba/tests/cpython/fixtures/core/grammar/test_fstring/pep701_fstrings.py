# RUN: parse

# PEP 701: Quote reuse in f-strings
s = f"{'hello'}"
s = f"result: {d['key']}"
s = f"{'{'}"

# PEP 701: Backslashes in f-string expressions
s = f"newline: {chr(10)}"
s = f"tab: {'\t'}"
s = f"escaped: {'\\n'}"

# PEP 701: Multi-line f-string expressions
result = f"value: {
    x + y
}"

s = f"mapped: {[
    item
    for item in range(10)
    if item > 5
]}"

# PEP 701: Deeply nested f-strings
s = f"{'hello':>10}"
s = f"{f"{f"deep"}"}"
s = f"{'='*40}"

# PEP 701: Lambda in f-string
s = f"{(lambda x: x + 1)(5)}"

# PEP 701: Dict comprehension in f-string
s = f"{ {k: v for k, v in items} }"
