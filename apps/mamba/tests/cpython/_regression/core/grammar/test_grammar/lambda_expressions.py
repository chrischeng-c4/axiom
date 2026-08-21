# RUN: parse
# CPython 3.12 test_grammar: lambda expressions

# Simple lambda
f = lambda: 42
g = lambda x: x + 1
h = lambda x, y: x + y

# Lambda with defaults
fn = lambda x, y=10: x + y

# Lambda with *args, **kwargs
var = lambda *args: args
kw = lambda **kwargs: kwargs
both = lambda *args, **kwargs: (args, kwargs)

# Nested lambda
compose = lambda f, g: lambda x: f(g(x))

# Lambda in expressions
pairs = [(1, "one"), (2, "two"), (3, "three")]
pairs.sort(key=lambda pair: pair[1])

# Lambda with conditional
abs_val = lambda x: x if x >= 0 else -x

# Lambda as default argument
def make_adder(n=lambda: 0):
    return n
