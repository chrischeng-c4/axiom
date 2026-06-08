# R1: Decorators with any expression in decorator position (PEP 614).
# The decorator may be a call expression (factory pattern).

def route(path):
    def decorator(func):
        return func
    return decorator

@route("/api")
def handler():
    return 200

print(handler())

# Chained: outer factory wraps with an arg, inner is a direct decorator.
call_count = 0

def counter(func):
    global call_count
    call_count = call_count + 1
    return func

def make_deco(label):
    return counter

@make_deco("v1")
def greet():
    return 42

print(call_count)
print(greet())
