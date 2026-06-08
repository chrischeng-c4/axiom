# Mamba compatibility test: what FastAPI ecosystem features work?
# Run with: cclab-mamba run test_mamba_compat.py

# --- Level 0: Pure Python constructs used by the app ---
print("=== Level 0: Pure Python ===")

# f-strings (just fixed!)
name = "Todo"
print(f"App name: {name}")

# Dataclass-like pattern
title = "Buy groceries"
completed = False
print(f"title={title}, completed={completed}")

# Dict operations (like JSON payloads)
todo = {"id": 1, "title": "Test item", "completed": False}
print(f"todo id: {todo['id']}")
todo["completed"] = True
print(f"toggled: {todo['completed']}")

# List operations (like query results)
todos = [
    {"id": 1, "title": "First"},
    {"id": 2, "title": "Second"},
    {"id": 3, "title": "Third"},
]
filtered = [t for t in todos if t["id"] > 1]
print(f"filtered count: {len(filtered)}")

# String operations (like URL parsing)
url = "/api/todos/42"
parts = url.split("/")
todo_id = int(parts[-1])
print(f"parsed id: {todo_id}")

print("Level 0: OK\n")

# --- Level 1: Class inheritance (like SQLAlchemy models) ---
print("=== Level 1: Classes ===")

class Base:
    pass

class Todo(Base):
    def __init__(self, id, title, completed):
        self.id = id
        self.title = title
        self.completed = completed

    def __repr__(self):
        return f"Todo(id={self.id}, title={self.title})"

item = Todo(1, "Test", False)
print(item)
item.completed = True
print(f"completed: {item.completed}")

print("Level 1: OK\n")

# --- Level 2: Decorators (like @app.get) ---
print("=== Level 2: Decorators ===")

routes = {}

def route(path):
    def decorator(func):
        routes[path] = func
        return func
    return decorator

@route("/api/todos")
def list_todos():
    return [{"id": 1, "title": "Test"}]

@route("/api/todos/{id}")
def get_todo():
    return {"id": 1, "title": "Test"}

print(f"registered routes: {len(routes)}")
result = routes["/api/todos"]()
print(f"list result: {result}")

print("Level 2: OK\n")

# --- Level 3: Async/await (like FastAPI handlers) ---
print("=== Level 3: Async ===")

async def async_handler():
    return {"status": "ok"}

import asyncio
result = asyncio.run(async_handler())
print(f"async result: {result}")

print("Level 3: OK\n")

# --- Level 4: Type hints / annotations (like Pydantic models) ---
print("=== Level 4: Type annotations ===")

class TodoCreate:
    title: str

    def __init__(self, title: str):
        self.title = title

tc = TodoCreate("New todo")
print(f"created: {tc.title}")

print("Level 4: OK\n")

# --- Level 5: Context managers (like DB sessions) ---
print("=== Level 5: Context managers ===")

class FakeSession:
    def __enter__(self):
        print("  session opened")
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        print("  session closed")
        return False

    def query(self):
        return [{"id": 1}]

with FakeSession() as db:
    result = db.query()
    print(f"  query result: {result}")

print("Level 5: OK\n")

# --- Level 6: Exception handling (like HTTP errors) ---
print("=== Level 6: Exceptions ===")

class HTTPException(Exception):
    def __init__(self, status_code, detail):
        self.status_code = status_code
        self.detail = detail

try:
    raise HTTPException(404, "Todo not found")
except HTTPException as e:
    print(f"caught: {e.status_code} {e.detail}")

print("Level 6: OK\n")

# --- Level 7: Generator/yield (like FastAPI Depends) ---
print("=== Level 7: Generators ===")

def get_db():
    print("  db connect")
    yield "db_session"
    print("  db disconnect")

gen = get_db()
session = next(gen)
print(f"  got session: {session}")
try:
    next(gen)
except StopIteration:
    pass

print("Level 7: OK\n")

print("=== All levels passed ===")
