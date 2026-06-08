from pydantic import BaseModel
from datetime import datetime
from typing import Optional


class TodoCreate(BaseModel):
    title: str


class TodoUpdate(BaseModel):
    title: Optional[str] = None
    completed: Optional[bool] = None


class TodoResponse(BaseModel):
    id: int
    title: str
    completed: bool
    created_at: datetime

    model_config = {"from_attributes": True}
