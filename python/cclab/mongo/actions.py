"""
Action decorators for document lifecycle events.

This module provides Beanie-compatible event hooks:
- before_event: Execute before an event (Insert, Replace, Save, Delete)
- after_event: Execute after an event

Example:
    >>> from cclab.mongo import Document
    >>> from cclab.actions import before_event, after_event, Insert, Save
    >>>
    >>> class User(Document):
    ...     email: str
    ...     created_at: datetime = None
    ...     updated_at: datetime = None
    ...
    ...     @before_event(Insert)
    ...     def set_created_at(self):
    ...         self.created_at = datetime.now()
    ...
    ...     @before_event(Save)
    ...     def set_updated_at(self):
    ...         self.updated_at = datetime.now()
    ...
    ...     @after_event(Insert)
    ...     async def send_welcome_email(self):
    ...         await send_email(self.email, "Welcome!")
"""

from __future__ import annotations

import asyncio
import functools
from enum import Enum, auto
from typing import Any, Callable, List, Optional, TYPE_CHECKING, Type, Union

if TYPE_CHECKING:
    from .document import Document


# ===================
# Event Types
# ===================


class EventType(Enum):
    """Document lifecycle events."""
    INSERT = auto()
    REPLACE = auto()
    SAVE = auto()  # Insert or Update
    DELETE = auto()
    VALIDATE_ON_SAVE = auto()


# Convenience aliases
Insert = EventType.INSERT
Replace = EventType.REPLACE
Save = EventType.SAVE
Delete = EventType.DELETE
ValidateOnSave = EventType.VALIDATE_ON_SAVE


# ===================
# Event Registry
# ===================


class ActionRegistry:
    """
    Registry for document lifecycle actions.

    Stores before/after handlers keyed by event type.
    Each Document class has its own registry.
    """

    def __init__(self) -> None:
        self._before: dict[EventType, List[Callable]] = {}
        self._after: dict[EventType, List[Callable]] = {}

    def register_before(self, event: EventType, handler: Callable) -> None:
        """Register a before-event handler."""
        if event not in self._before:
            self._before[event] = []
        self._before[event].append(handler)

    def register_after(self, event: EventType, handler: Callable) -> None:
        """Register an after-event handler."""
        if event not in self._after:
            self._after[event] = []
        self._after[event].append(handler)

    def get_before_handlers(self, event: EventType) -> List[Callable]:
        """Get all before handlers for an event."""
        handlers = list(self._before.get(event, []))
        # Save event includes Insert and Replace handlers
        if event == EventType.SAVE:
            handlers.extend(self._before.get(EventType.INSERT, []))
            handlers.extend(self._before.get(EventType.REPLACE, []))
        return handlers

    def get_after_handlers(self, event: EventType) -> List[Callable]:
        """Get all after handlers for an event."""
        handlers = list(self._after.get(event, []))
        # Save event includes Insert and Replace handlers
        if event == EventType.SAVE:
            handlers.extend(self._after.get(EventType.INSERT, []))
            handlers.extend(self._after.get(EventType.REPLACE, []))
        return handlers


# Global registry storage (per Document class)
_registries: dict[type, ActionRegistry] = {}


def get_registry(document_class: type) -> ActionRegistry:
    """Get or create the action registry for a document class."""
    if document_class not in _registries:
        _registries[document_class] = ActionRegistry()
    return _registries[document_class]


# ===================
# Event Decorators
# ===================


def before_event(*events: EventType) -> Callable:
    """
    Decorator to run a method before specified events.

    The decorated method can be sync or async.
    It receives the document instance as `self`.

    Args:
        *events: One or more EventType values

    Example:
        >>> @before_event(Insert)
        ... def set_created_at(self):
        ...     self.created_at = datetime.now()

        >>> @before_event(Insert, Replace)
        ... async def validate_email(self):
        ...     if not is_valid_email(self.email):
        ...         raise ValueError("Invalid email")
    """
    def decorator(func: Callable) -> Callable:
        # Mark the function with event metadata
        if not hasattr(func, "_before_events"):
            func._before_events = []
        func._before_events.extend(events)
        return func
    return decorator


def after_event(*events: EventType) -> Callable:
    """
    Decorator to run a method after specified events.

    The decorated method can be sync or async.
    It receives the document instance as `self`.

    Args:
        *events: One or more EventType values

    Example:
        >>> @after_event(Insert)
        ... async def send_notification(self):
        ...     await notify_admin(f"New user: {self.email}")

        >>> @after_event(Delete)
        ... def log_deletion(self):
        ...     logging.info(f"Deleted user: {self.id}")
    """
    def decorator(func: Callable) -> Callable:
        # Mark the function with event metadata
        if not hasattr(func, "_after_events"):
            func._after_events = []
        func._after_events.extend(events)
        return func
    return decorator


# ===================
# Action Execution
# ===================


async def run_before_event(document: "Document", event: EventType) -> None:
    """
    Run all before-event handlers for a document.

    Args:
        document: The document instance
        event: The event type
    """
    # Fast path: Skip expensive iteration if no hooks exist
    # Cache hook existence per class to avoid repeated dir() calls
    doc_class = document.__class__
    if not hasattr(doc_class, '_has_before_hooks_cache'):
        # First time checking this class - scan for hooks
        has_hooks = False
        for method_name in dir(document):
            if method_name.startswith("_"):
                continue
            method = getattr(document, method_name, None)
            if method and callable(method) and hasattr(method, "_before_events"):
                has_hooks = True
                break
        doc_class._has_before_hooks_cache = has_hooks

    # Early return if no hooks registered for this class
    if not doc_class._has_before_hooks_cache:
        return

    # Collect handlers from class methods with decorators
    for method_name in dir(document):
        if method_name.startswith("_"):
            continue
        method = getattr(document, method_name, None)
        if method is None or not callable(method):
            continue

        # Check for before_event decorator
        if hasattr(method, "_before_events") and event in method._before_events:
            result = method()
            if asyncio.iscoroutine(result):
                await result


async def run_after_event(document: "Document", event: EventType) -> None:
    """
    Run all after-event handlers for a document.

    Args:
        document: The document instance
        event: The event type
    """
    # Fast path: Skip expensive iteration if no hooks exist
    # Cache hook existence per class to avoid repeated dir() calls
    doc_class = document.__class__
    if not hasattr(doc_class, '_has_after_hooks_cache'):
        # First time checking this class - scan for hooks
        has_hooks = False
        for method_name in dir(document):
            if method_name.startswith("_"):
                continue
            method = getattr(document, method_name, None)
            if method and callable(method) and hasattr(method, "_after_events"):
                has_hooks = True
                break
        doc_class._has_after_hooks_cache = has_hooks

    # Early return if no hooks registered for this class
    if not doc_class._has_after_hooks_cache:
        return

    # Collect handlers from class methods with decorators
    for method_name in dir(document):
        if method_name.startswith("_"):
            continue
        method = getattr(document, method_name, None)
        if method is None or not callable(method):
            continue

        # Check for after_event decorator
        if hasattr(method, "_after_events") and event in method._after_events:
            result = method()
            if asyncio.iscoroutine(result):
                await result


# ===================
# Validation Actions
# ===================


async def run_validate_on_save(document: "Document") -> None:
    """
    Run validation before save operations.

    This is triggered for Insert, Replace, and Save events when
    use_validation is enabled in Settings.
    """
    # Check if document has validation settings
    settings = getattr(document, "Settings", None)
    if settings and getattr(settings, "use_validation", False):
        await run_before_event(document, EventType.VALIDATE_ON_SAVE)


# ===================
# Exports
# ===================

__all__ = [
    # Event types
    "EventType",
    "Insert",
    "Replace",
    "Save",
    "Delete",
    "ValidateOnSave",
    # Decorators
    "before_event",
    "after_event",
    # Utilities
    "run_before_event",
    "run_after_event",
    "run_validate_on_save",
    "ActionRegistry",
    "get_registry",
]
