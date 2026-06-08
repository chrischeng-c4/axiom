# Stacked decorators: applied bottom-up

def bold(func):
    def wrapper(*args, **kwargs):
        return "<b>" + func(*args, **kwargs) + "</b>"
    return wrapper

def italic(func):
    def wrapper(*args, **kwargs):
        return "<i>" + func(*args, **kwargs) + "</i>"
    return wrapper

@bold
@italic
def greet(name):
    return f"Hello, {name}"

# italic applied first, then bold wraps the result
print(greet("World"))

# Single decorator
@bold
def simple():
    return "text"

print(simple())
