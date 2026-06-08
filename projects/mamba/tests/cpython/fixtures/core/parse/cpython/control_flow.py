# RUN: parse
# CPython-derived: control flow (if, while, for, try, with)

# --- if / elif / else ---
if x > 0:
    pass
elif x == 0:
    pass
else:
    pass

# --- nested if ---
if a:
    if b:
        pass
    else:
        pass

# --- while ---
while i < 10:
    i += 1

# --- for ---
for item in items:
    pass

# --- for with type annotation ---
for i: int in range(10):
    pass

# --- nested for ---
for i in outer:
    for j in inner:
        pass

# --- try / except ---
try:
    pass
except ValueError:
    pass

# --- try / except with binding ---
try:
    x = 1
except ValueError as e:
    pass

# --- try / except / else / finally ---
try:
    x = 1
except TypeError:
    pass
except ValueError as e:
    pass
else:
    pass
finally:
    pass

# --- bare except ---
try:
    pass
except:
    pass

# --- with statement ---
with open("file") as f:
    pass

# --- with multiple items ---
with open("a") as f, open("b") as g:
    pass

# --- nested control flow ---
for i in items:
    if i > 0:
        while i > 0:
            i -= 1
    else:
        continue
