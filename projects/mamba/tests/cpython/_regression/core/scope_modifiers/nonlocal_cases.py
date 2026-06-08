# nonlocal / global edge cases

# Counter closure via nonlocal
def make_counter():
    count = 0
    def inc():
        nonlocal count
        count = count + 1
        return count
    return inc

c = make_counter()
print(c())
print(c())
print(c())
