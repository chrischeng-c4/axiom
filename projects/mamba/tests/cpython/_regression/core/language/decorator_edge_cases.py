# Decorator edge cases

# Stacked decorators — bottom-up application
def d1(f):
    print('d1 applied')
    return f

def d2(f):
    print('d2 applied')
    return f

@d1
@d2
def foo():
    pass

# Parameterized decorator
def repeat(n):
    def decorator(f):
        def wrapper():
            for _ in range(n):
                f()
        return wrapper
    return decorator

@repeat(3)
def greet():
    print('hi')

greet()
