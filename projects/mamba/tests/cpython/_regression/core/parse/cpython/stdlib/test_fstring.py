# RUN: parse
# Extracted from CPython 3.12 Lib/test/test_fstring.py — f-string syntax constructs only.
import decimal

# --- Basic f-strings ---
x = f''
x = f""
x = f"no-braces"
x = f'no-braces'
x = f"""no-braces"""
x = f'''no-braces'''
x = f' '
x = f'\n'
x = f'\r'

# --- Simple interpolation ---
a = 10
x = f'{a}'
x = f'{a!r}'
x = f'{a!s}'
x = f'{a!a}'
x = f'result: {a}'
x = f'{a} result'
x = f'x={a} y={a}'

# --- Integer and float literals ---
x = f'{10}'
x = f'{0}'
x = f'{-1}'
x = f'{1.5}'
x = f'{3.14159}'

# --- String expressions ---
x = f'{"abc"}'
x = f"{'abc'}"
x = f'{"abc"!r}'
x = f'{"abc"!s}'
x = f'{"abc"!a}'

# --- Escaped braces ---
x = f'{{}}'
x = f'{{a}}'
x = f'a{{b}}c'
x = f'{{{10}}}'
x = f'{{{{10}}}}'

# --- Format spec ---
x = f'{10:d}'
x = f'{10:04d}'
x = f'{10:#010x}'
x = f'{3.14:.2f}'
x = f'{3.14:10.2f}'
x = f'{10:+d}'
x = f'{10: d}'
x = f'{10:=+10d}'
x = f'{"hello":>10}'
x = f'{"hello":<10}'
x = f'{"hello":^10}'
x = f'{"hello":*^10}'
x = f'{1000:,}'
x = f'{1000:_}'

# --- Dynamic format spec (nested f-string in format spec) ---
width = 10
precision = 4
x = f'{"hello":>{width}}'
x = f'{3.14:.{precision}f}'
x = f'{3.14:{width}.{precision}f}'

# --- Conversion flags with format spec ---
x = f'{a!r:>20}'
x = f'{a!s:>20}'
x = f'{a!a:>20}'

# --- Arithmetic expressions ---
x = f'{1+1}'
x = f'{3*4}'
x = f'{10//3}'
x = f'{10%3}'
x = f'{2**8}'
x = f'{10-3}'
x = f'{-a}'
x = f'{+a}'
x = f'{~0}'

# --- Comparison expressions ---
x = f'{a == 10}'
x = f'{a != 10}'
x = f'{a > 5}'
x = f'{a < 20}'
x = f'{a >= 10}'
x = f'{a <= 10}'

# --- Boolean expressions ---
x = f'{True and False}'
x = f'{True or False}'
x = f'{not True}'
x = f'{a and a}'
x = f'{a or 0}'

# --- Ternary / conditional expression ---
x = f'{a if a > 5 else 0}'
x = f'{"yes" if True else "no"}'
x = f'{"positive" if a > 0 else "negative" if a < 0 else "zero"}'

# --- Function calls ---
x = f'{len("abc")}'
x = f'{str(10)}'
x = f'{repr(a)}'
x = f'{int("10")}'
x = f'{max(1, 2, 3)}'
x = f'{min(1, 2, 3)}'
x = f'{sum([1, 2, 3])}'
x = f'{abs(-5)}'
x = f'{round(3.14159, 2)}'
x = f'{type(a).__name__}'

# --- Method calls ---
x = f'{"hello".upper()}'
x = f'{"HELLO".lower()}'
x = f'{"hello world".title()}'
x = f'{"hello".replace("l", "r")}'
x = f'{"a,b,c".split(",")}'
x = f'{" hello ".strip()}'
x = f'{"hello".center(20)}'
x = f'{"hello".zfill(10)}'
x = f'{"hello".encode()}'

# --- Subscript / index access ---
d = {'key': 'value', 0: 'zero'}
lst = [1, 2, 3]
x = f'{lst[0]}'
x = f'{lst[-1]}'
x = f'{lst[0:2]}'
x = f'{d["key"]}'
x = f"{d['key']}"
x = f'{d[0]}'

# --- Attribute access ---
x = f'{"hello".__class__}'
x = f'{"hello".__class__.__name__}'
x = f'{a.__class__}'

# --- Nested f-strings ---
x = f'{f"{a}"}'
x = f'{f"{f"{a}"}"}'
x = f'outer {f"inner {a}"} end'
x = f'{f"{a}" + f"{a}"}'

# --- Concatenated f-strings ---
x = f'hello' f' world'
x = f'{a}' f'{a}'
x = f'x' 'y'
x = 'x' f'y'
x = f'{a}' ' literal'
x = ' literal' f'{a}'
x = f'a' f'b' f'c'

# --- Raw f-strings (rf / fr) ---
x = rf'\n{a}'
x = fr'\n{a}'
x = rf'{a}\n'
x = Rf'{a}'
x = fR'{a}'
x = FR'{a}'
x = rF'{a}'

# --- Triple-quoted f-strings ---
x = f"""hello {a}"""
x = f'''hello {a}'''
x = f"""
multi
line
{a}
"""
x = f'''
multi
line
{a}
'''

# --- Multiline expressions inside f-strings ---
x = f'{(
    a
)}'
x = f'{(
    a +
    1
)}'

# --- List / dict / set / tuple in f-strings ---
x = f'{[1, 2, 3]}'
x = f'{(1, 2, 3)}'
x = f'{{1, 2, 3}}'
x = f'{{"a": 1, "b": 2}}'
x = f'{dict(a=1)}'

# --- Comprehensions in f-strings ---
x = f'{[i for i in range(5)]}'
x = f'{[i*2 for i in range(5)]}'
x = f'{[i for i in range(10) if i % 2 == 0]}'
x = f'{{i: i*2 for i in range(5)}}'
x = f'{{i for i in range(5)}}'
x = f'{tuple(i for i in range(5))}'
x = f'{sum(i for i in range(10))}'

# --- Lambda in f-strings ---
x = f'{(lambda x: x + 1)(2)}'
x = f'{(lambda: 42)()}'
x = f'{(lambda x, y: x + y)(1, 2)}'

# --- Walrus operator in f-strings ---
x = f'{(y := 10)}'
x = f'{(z := a + 1)}'

# --- Complex nested expressions ---
x = f'{"x" * 10}'
x = f'{"hello"!r:>20}'

# --- Chained attribute/subscript ---
x = f'{[1, 2, 3][0]}'
x = f'{"hello"[0]}'
x = f'{"hello"[1:3]}'
x = f'{"hello"[::-1]}'

# --- Star expressions and unpacking in calls inside f-strings ---
x = f'{",".join(["a", "b", "c"])}'
x = f'{",".join(str(i) for i in range(5))}'

# NOTE: PEP 701 (Python 3.12) features not yet supported by parser:
# - Same-quote reuse in f-strings: f"{'hello'}" with double quotes inside
# - Deeply nested same-quote f-strings
# - Comments inside f-string expressions
# - Backslash sequences inside f-string expressions

# --- Unicode in f-strings ---
x = f'\u0041'
x = f'\U00000041'
x = f'\N{LATIN SMALL LETTER A}'
x = f'\x41'

# --- Decimal formatting ---
x = f'{decimal.Decimal("3.14"):.2f}'
x = f'{decimal.Decimal("1000"):,}'

# --- Boolean and None formatting ---
x = f'{True}'
x = f'{False}'
x = f'{True!r}'
x = f'{True!s}'
x = f'{None}'
x = f'{None!r}'
x = f'{None!s}'

# --- Bytes repr ---
x = f'{b"hello"!r}'
x = f'{b"hello"!a}'

# --- Complex numbers ---
x = f'{1+2j}'
x = f'{complex(1, 2)}'
x = f'{(1+2j)!r}'

# --- Whitespace around expression ---
x = f'{ a }'
x = f'{  a  }'

# --- Nested braces and dynamic format spec ---
x = f'{"{"}'
x = f'{"}"}'
x = f'{10:{"d"}}'
x = f'{10:{"04d"}}'

# --- Yield in f-string inside generator ---
def g():
    x = f'{(yield 1)}'

# --- Async expressions in f-strings ---
async def af():
    import asyncio
    x = f'{await asyncio.sleep(0)}'

# --- Class __format__ protocol ---
class MyClass:
    def __repr__(self):
        return 'MyClass()'
    def __str__(self):
        return 'my class'
    def __format__(self, spec):
        return f'formatted({spec})'

obj = MyClass()
x = f'{obj}'
x = f'{obj!r}'
x = f'{obj!s}'
x = f'{obj!a}'
x = f'{obj:spec}'

# --- Star unpacking inside f-string function calls ---
args = [1, 2, 3]
kwargs = {'sep': ','}
x = f'{max(*args)}'
x = f'{dict(**kwargs)}'

# --- Nested dictionary access ---
data = {'a': {'b': {'c': 42}}}
x = f'{data["a"]["b"]["c"]}'

# --- Multiple expressions in one f-string ---
y = 20
x = f'{a} + {y} = {a + y}'
x = f'({a}, {y})'
x = f'[{a}:{y}]'
x = f'{a!r} vs {y!s}'

# --- F-string with no interpolation (just literal) ---
x = f'just a string'
x = f"just a string"
x = f"""just a string"""
x = f'''just a string'''

# --- F-string in various statement contexts ---
for i in range(3):
    x = f'item {i}'

while False:
    x = f'{a}'
    break

if f'{a}' == '10':
    pass

[f'{i}' for i in range(3)]
{f'{i}' for i in range(3)}
{f'{i}': i for i in range(3)}

# --- F-string as default argument ---
def func(x=f'{10}'):
    pass

# --- F-string in return ---
def func2():
    return f'{a}'

# --- F-string as decorator argument ---
def decorator(name):
    def wrapper(fn):
        return fn
    return wrapper

@decorator(f'name_{a}')
def decorated():
    pass

# --- F-string in assert ---
val = 10
assert val == 10, f'Expected 10, got {val}'

# --- F-string in raise ---
try:
    raise ValueError(f'bad value: {val}')
except ValueError:
    pass

# --- Additional format spec patterns ---
x = f'{3.14:+.2f}'
x = f'{1000:>10,}'
x = f'{255:#x}'
x = f'{255:#o}'
x = f'{255:#b}'
x = f'{42:c}'

# --- Tuple display (needs parens) ---
x = f'{(1, 2)}'
x = f'{(1,)}'
x = f'{()}'

# --- Mixed string prefix concatenation ---
x = 'a' f'b{a}' 'c'
x = f'{a}' + 'literal'

# --- End of f-string constructs ---
