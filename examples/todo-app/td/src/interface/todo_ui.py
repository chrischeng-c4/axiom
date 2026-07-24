"""FocusFlow UI tech design: parsed by aw, never imported at runtime."""

from aw.ui import Event, Slot, component, page, token
from domain.todo import Todo, TodoId, TodoInput

token("color.brand", "#b9f2dc", "color")
token("color.surface", "#ffffff", "color")
token("space.page", "3.5rem", "dimension")
token("motion.feedback", "180ms ease", "duration")


@component("Task filters and current selection")
def TaskFilterNav(filters: list[str], selected: str, on_select: Event[str]): ...


@component("Create task with priority and due date")
def TaskComposer(on_submit: Event[TodoInput]): ...


@component("One task row with completion and deletion actions")
def TaskRow(todo: Todo, on_toggle: Event[TodoId], on_delete: Event[TodoId]): ...


@component("Task collection with an explicit item slot")
def TaskList(todos: list[Todo], item: Slot[TaskRow]): ...


@component("Completion count and ratio")
def CompletionProgress(todos: list[Todo]): ...


@page
def TodoPage(todos: list[Todo]):
    return AppShell(
        sidebar=TaskFilterNav(filters=["all", "open", "done"], selected="all"),
        main=Stack(
            TaskComposer(),
            TaskList(todos=todos, item=TaskRow()),
            CompletionProgress(todos=todos),
        ),
    )
