# RUN: parse
# Operator precedence comprehensive test fixture (#570)

# --- arithmetic precedence ---
x = 2 + 3 * 4
x = (2 + 3) * 4
x = 2 ** 3 ** 2
x = -2 ** 2
x = -(2 ** 2)
x = 2 + 3 * 4 - 5 / 2
x = 2 * 3 + 4 * 5
x = 10 // 3 % 2
x = 1 + 2 * 3 ** 4

# --- unary operator precedence ---
x = -1
x = +1
x = ~0
x = not True
x = not not True
x = -(-1)
x = ~(~0)
x = -(+x)
x = +(-x)

# --- comparison chaining ---
x = 1 < 2 < 3
x = 1 < 2 <= 3
x = 1 <= 2 < 3 <= 4
x = 1 == 1 == 1
x = 1 != 2 != 3
x = 1 < 2 > 0
x = 1 is 1 is 1
x = 1 is not 2 is not 3
x = 1 in [1] in [[1]]
x = 1 not in [2] not in [[3]]

# --- boolean operator precedence ---
x = True and False or True
x = True or False and True
x = not True and False
x = not (True and False)
x = True and not False
x = True or True and False
x = (True or True) and False

# --- bitwise operator precedence ---
x = 1 | 2 & 3
x = (1 | 2) & 3
x = 1 ^ 2 | 3
x = 1 ^ (2 | 3)
x = 1 & 2 ^ 3 | 4
x = ~1 & 2
x = ~(1 & 2)
x = 1 << 2 | 3
x = 1 | 2 << 3
x = 1 << 2 + 3
x = (1 << 2) + 3

# --- mixed arithmetic and bitwise ---
x = 2 + 3 & 4
x = (2 + 3) & 4
x = 2 * 3 | 4
x = 2 | 3 * 4
x = 1 + 2 << 3
x = (1 + 2) << 3
x = 1 << 2 + 3

# --- ternary operator precedence ---
x = 1 if True else 2
x = 1 if True else 2 if False else 3
x = (1 if True else 2) if False else 3
x = 1 + 2 if True else 3 + 4
x = (1 + 2) if True else (3 + 4)

# --- walrus operator precedence ---
x = (y := 1 + 2)
x = (y := 1 if True else 2)

# --- lambda precedence ---
f = lambda: 1 + 2
f = lambda x: x * 2 + 1
f = lambda x, y: x if x > y else y
f = lambda: (yield 1)  # inside generator context

# --- call / attribute / subscript precedence ---
x = a.b.c
x = a.b()
x = a().b
x = a[0].b
x = a.b[0]
x = a[0][1]
x = a()()
x = a.b().c[0]
x = -a.b
x = (-a).b
x = not a.b
x = a ** b.c

# --- comparison with other operators ---
x = 1 + 2 == 3
x = 1 == 2 + 1
x = 1 + 2 < 3 + 4
x = 1 & 2 == 0
x = 1 == 0 & 2
x = 1 | 2 > 0

# --- assignment expression contexts ---
x = [y for y in range(10) if y > 5]
x = {k: v for k, v in enumerate(range(5))}
x = (a for a in range(10) if a % 2 == 0)

# --- complex nested expressions ---
x = (1 + 2) * (3 - 4) / (5 + 6)
x = ((a + b) * c - d) ** (e + f)
x = a if (b > c and d < e) or f else g
x = [x ** 2 for x in range(10) if x % 2 == 0]
x = {k: v for k, v in zip(range(5), range(5, 10)) if k != 2}

# --- power associativity (right-to-left) ---
x = 2 ** 3 ** 2
x = (2 ** 3) ** 2

# --- unary minus with power ---
x = -1 ** 2
x = (-1) ** 2
x = -1 ** -1
