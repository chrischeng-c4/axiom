# scope broad

# global assignment
total = 0

def add_to_total(n):
    global total
    total += n

add_to_total(5)
add_to_total(10)
print(total)

# local shadowing
def shadow():
    x = 1
    def inner():
        x = 2
        return x
    return x, inner()

print(shadow())

# function-level scope
def using_var():
    total = 0
    for i in range(10):
        total += i
    return total

print(using_var())

# nested function simple (read-only outer)
def outer():
    x = 10
    def inner():
        return x * 2
    return inner()

print(outer())

# class-level scope
class Config:
    DEFAULT = 42
    OPTIONS = [1, 2, 3]

print(Config.DEFAULT)
print(Config.OPTIONS)
Config.DEFAULT = 100
print(Config.DEFAULT)
