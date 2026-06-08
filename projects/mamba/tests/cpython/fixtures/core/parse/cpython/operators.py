# RUN: parse
# CPython-derived: all operator types

# --- arithmetic ---
a = 1 + 2
a = 3 - 1
a = 2 * 3
a = 10 / 3
a = 10 // 3
a = 10 % 3
a = 2 ** 10

# --- comparison ---
a = 1 == 1
a = 1 != 2
a = 1 < 2
a = 2 > 1
a = 1 <= 2
a = 2 >= 1

# --- logical ---
a = True and False
a = True or False
a = not True

# --- bitwise ---
a = 0xFF & 0x0F
a = 0x0F | 0xF0
a = 0xFF ^ 0x0F
a = ~0xFF
a = 1 << 4
a = 16 >> 2

# --- identity ---
a = x is None
a = x is not None

# --- membership ---
a = 1 in items
a = 1 not in items

# --- operator precedence ---
a = 1 + 2 * 3
a = (1 + 2) * 3
a = 2 ** 3 ** 2
a = 1 + 2 > 2 and 3 < 4
a = not (a and b) or c

# --- matrix multiply (PEP 465) ---
a = x @ y
