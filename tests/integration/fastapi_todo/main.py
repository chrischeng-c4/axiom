import os
from contextlib import asynccontextmanager

from fastapi import FastAPI, Depends, HTTPException
from fastapi.responses import FileResponse
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from database import init_db, get_db
from models import Todo
from schemas import TodoCreate, TodoUpdate, TodoResponse

STATIC_DIR = os.path.join(os.path.dirname(__file__), "static")


@asynccontextmanager
async def lifespan(app: FastAPI):
    await init_db()
    yield


app = FastAPI(title="Mamba Todo", lifespan=lifespan)


# --- Frontend ---

@app.get("/")
async def index():
    return FileResponse(os.path.join(STATIC_DIR, "index.html"))


# --- API ---

@app.get("/api/todos", response_model=list[TodoResponse])
async def list_todos(db: AsyncSession = Depends(get_db)):
    result = await db.execute(select(Todo).order_by(Todo.created_at.desc()))
    return result.scalars().all()


@app.post("/api/todos", response_model=TodoResponse, status_code=201)
async def create_todo(todo: TodoCreate, db: AsyncSession = Depends(get_db)):
    item = Todo(title=todo.title)
    db.add(item)
    await db.commit()
    await db.refresh(item)
    return item


@app.patch("/api/todos/{todo_id}", response_model=TodoResponse)
async def update_todo(todo_id: int, todo: TodoUpdate, db: AsyncSession = Depends(get_db)):
    result = await db.execute(select(Todo).where(Todo.id == todo_id))
    item = result.scalar_one_or_none()
    if not item:
        raise HTTPException(status_code=404, detail="Todo not found")
    if todo.title is not None:
        item.title = todo.title
    if todo.completed is not None:
        item.completed = todo.completed
    await db.commit()
    await db.refresh(item)
    return item


@app.delete("/api/todos/{todo_id}", status_code=204)
async def delete_todo(todo_id: int, db: AsyncSession = Depends(get_db)):
    result = await db.execute(select(Todo).where(Todo.id == todo_id))
    item = result.scalar_one_or_none()
    if not item:
        raise HTTPException(status_code=404, detail="Todo not found")
    await db.delete(item)
    await db.commit()
