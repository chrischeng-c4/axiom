# RUN: parse
# CPython 3.12 test_decorators: class decorators

def singleton(cls):
    instances = {}
    def get_instance(*args, **kwargs):
        if cls not in instances:
            instances[cls] = cls(*args, **kwargs)
        return instances[cls]
    return get_instance

@singleton
class Database:
    def __init__(self):
        self.connected = False

# Decorator with arguments on class
def register(name):
    def decorator(cls):
        cls.registry_name = name
        return cls
    return decorator

@register("my_service")
class Service:
    pass

# Stacked class decorators
def add_repr(cls):
    cls.__repr__ = lambda self: f"{cls.__name__}()"
    return cls

@singleton
@add_repr
class Config:
    pass
