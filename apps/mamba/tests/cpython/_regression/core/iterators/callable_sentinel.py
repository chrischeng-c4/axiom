# iter(callable, sentinel) — two-argument form with named functions

# Basic: stops when callable returns sentinel
count = 0
def counter():
    global count
    count += 1
    return count

print(list(iter(counter, 4)))
