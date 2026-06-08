"""
Event system for data-bridge PostgreSQL ORM.

Provides SQLAlchemy-style event hooks for:
- CRUD operations (before/after insert, update, delete)
- Session events (before_flush, after_commit)
- Attribute changes

Example:
    from cclab.pg.events import event, listens_for

    @listens_for(User, 'before_insert')
    def set_created_at(mapper, connection, target):
        target.created_at = datetime.now()

    @listens_for(User, 'after_update')
    def log_update(mapper, connection, target):
        print(f"Updated user {target.id}")
"""
from __future__ import annotations

from enum import Enum, auto
from typing import (
    Any, Callable, Dict, List, Optional, Set, Tuple, Type, TypeVar,
    Union, TYPE_CHECKING
)
from dataclasses import dataclass, field
from functools import wraps
import weakref

if TYPE_CHECKING:
    from .table import Table

T = TypeVar('T')


class EventType(Enum):
    """Types of ORM events."""
    # CRUD events
    BEFORE_INSERT = "before_insert"
    AFTER_INSERT = "after_insert"
    BEFORE_UPDATE = "before_update"
    AFTER_UPDATE = "after_update"
    BEFORE_DELETE = "before_delete"
    AFTER_DELETE = "after_delete"

    # Session events
    BEFORE_FLUSH = "before_flush"
    AFTER_FLUSH = "after_flush"
    AFTER_COMMIT = "after_commit"
    AFTER_ROLLBACK = "after_rollback"

    # Attribute events
    ATTRIBUTE_SET = "attribute_set"
    ATTRIBUTE_REMOVE = "attribute_remove"

    # Load events
    AFTER_LOAD = "after_load"
    BEFORE_EXPIRE = "before_expire"


# Type alias for event listeners
EventListener = Callable[..., Optional[Any]]


@dataclass
class EventRegistration:
    """Represents a registered event listener."""
    event_type: EventType
    target: Optional[Type]  # None means global listener
    listener: EventListener
    propagate: bool = True
    once: bool = False
    priority: int = 0
    _called: bool = field(default=False, repr=False)


class EventDispatcher:
    """
    Central event dispatcher for ORM events.

    Manages registration and dispatching of event listeners.
    """

    _instance: Optional['EventDispatcher'] = None

    def __new__(cls) -> 'EventDispatcher':
        """Singleton pattern for global event dispatcher."""
        if cls._instance is None:
            cls._instance = super().__new__(cls)
            cls._instance._initialized = False
        return cls._instance

    def __init__(self):
        if self._initialized:
            return
        self._initialized = True

        # Global listeners: event_type -> list of registrations
        self._global_listeners: Dict[EventType, List[EventRegistration]] = {}

        # Per-target listeners: (target_class, event_type) -> list of registrations
        self._target_listeners: Dict[Tuple[Type, EventType], List[EventRegistration]] = {}

    def listen(
        self,
        target: Optional[Type],
        event_type: Union[EventType, str],
        listener: EventListener,
        propagate: bool = True,
        once: bool = False,
        priority: int = 0,
    ) -> EventRegistration:
        """
        Register an event listener.

        Args:
            target: Target class to listen on, or None for global
            event_type: Type of event to listen for
            listener: Callback function
            propagate: Whether to propagate to subclasses
            once: Whether to only fire once
            priority: Higher priority listeners fire first

        Returns:
            EventRegistration for later removal
        """
        if isinstance(event_type, str):
            event_type = EventType(event_type)

        registration = EventRegistration(
            event_type=event_type,
            target=target,
            listener=listener,
            propagate=propagate,
            once=once,
            priority=priority,
        )

        if target is None:
            # Global listener
            if event_type not in self._global_listeners:
                self._global_listeners[event_type] = []
            self._global_listeners[event_type].append(registration)
            self._global_listeners[event_type].sort(key=lambda r: -r.priority)
        else:
            # Per-target listener
            key = (target, event_type)
            if key not in self._target_listeners:
                self._target_listeners[key] = []
            self._target_listeners[key].append(registration)
            self._target_listeners[key].sort(key=lambda r: -r.priority)

        return registration

    def remove(self, registration: EventRegistration) -> bool:
        """
        Remove an event listener.

        Args:
            registration: Registration to remove

        Returns:
            True if removed, False if not found
        """
        if registration.target is None:
            listeners = self._global_listeners.get(registration.event_type, [])
            if registration in listeners:
                listeners.remove(registration)
                return True
        else:
            key = (registration.target, registration.event_type)
            listeners = self._target_listeners.get(key, [])
            if registration in listeners:
                listeners.remove(registration)
                return True
        return False

    def dispatch(
        self,
        event_type: Union[EventType, str],
        target_instance: Any,
        *args,
        **kwargs,
    ) -> List[Any]:
        """
        Dispatch an event to all registered listeners.

        Args:
            event_type: Type of event
            target_instance: Instance that triggered the event
            *args, **kwargs: Arguments to pass to listeners

        Returns:
            List of return values from listeners
        """
        if isinstance(event_type, str):
            event_type = EventType(event_type)

        results = []
        target_class = type(target_instance)

        # Get all applicable listeners
        listeners = self._get_listeners(target_class, event_type)

        # Track listeners to remove (for once=True)
        to_remove = []

        for reg in listeners:
            try:
                # Call listener with standard arguments
                result = reg.listener(target_instance, *args, **kwargs)
                results.append(result)

                if reg.once:
                    reg._called = True
                    to_remove.append(reg)

            except Exception as e:
                # Log error but don't stop other listeners
                import warnings
                warnings.warn(f"Event listener error: {e}")

        # Remove one-time listeners
        for reg in to_remove:
            self.remove(reg)

        return results

    async def dispatch_async(
        self,
        event_type: Union[EventType, str],
        target_instance: Any,
        *args,
        **kwargs,
    ) -> List[Any]:
        """
        Dispatch an event asynchronously.

        Handles both sync and async listeners.
        """
        import asyncio

        if isinstance(event_type, str):
            event_type = EventType(event_type)

        results = []
        target_class = type(target_instance)

        listeners = self._get_listeners(target_class, event_type)
        to_remove = []

        for reg in listeners:
            try:
                result = reg.listener(target_instance, *args, **kwargs)

                # Handle async listeners
                if asyncio.iscoroutine(result):
                    result = await result

                results.append(result)

                if reg.once:
                    reg._called = True
                    to_remove.append(reg)

            except Exception as e:
                import warnings
                warnings.warn(f"Event listener error: {e}")

        for reg in to_remove:
            self.remove(reg)

        return results

    def _get_listeners(
        self,
        target_class: Type,
        event_type: EventType,
    ) -> List[EventRegistration]:
        """Get all listeners applicable to a target class and event type."""
        listeners = []

        # Add global listeners first
        listeners.extend(self._global_listeners.get(event_type, []))

        # Add target-specific listeners (including inherited)
        for cls in target_class.__mro__:
            key = (cls, event_type)
            for reg in self._target_listeners.get(key, []):
                if reg.propagate or cls is target_class:
                    listeners.append(reg)

        # Sort by priority (already sorted per-list, but need global sort)
        listeners.sort(key=lambda r: -r.priority)

        return listeners

    def has_listeners(
        self,
        target_class: Optional[Type],
        event_type: Union[EventType, str],
    ) -> bool:
        """Check if any listeners are registered for an event."""
        if isinstance(event_type, str):
            event_type = EventType(event_type)

        if event_type in self._global_listeners and self._global_listeners[event_type]:
            return True

        if target_class is not None:
            for cls in target_class.__mro__:
                key = (cls, event_type)
                if key in self._target_listeners and self._target_listeners[key]:
                    return True

        return False

    def clear(self, target: Optional[Type] = None) -> None:
        """
        Clear event listeners.

        Args:
            target: If provided, only clear listeners for this target.
                   If None, clear all listeners.
        """
        if target is None:
            self._global_listeners.clear()
            self._target_listeners.clear()
        else:
            # Clear only for specific target
            keys_to_remove = [
                key for key in self._target_listeners
                if key[0] is target
            ]
            for key in keys_to_remove:
                del self._target_listeners[key]


# Global event dispatcher instance
event = EventDispatcher()


def listens_for(
    target: Optional[Type],
    event_type: Union[EventType, str],
    propagate: bool = True,
    once: bool = False,
    priority: int = 0,
) -> Callable[[EventListener], EventListener]:
    """
    Decorator to register an event listener.

    Example:
        @listens_for(User, 'before_insert')
        def set_defaults(target):
            target.created_at = datetime.now()

        @listens_for(None, 'after_commit')  # Global listener
        def log_commit(target):
            print("Committed!")

    Args:
        target: Target class, or None for global listener
        event_type: Event type to listen for
        propagate: Whether to fire for subclasses
        once: Whether to only fire once
        priority: Higher priority listeners fire first

    Returns:
        Decorator function
    """
    def decorator(fn: EventListener) -> EventListener:
        event.listen(target, event_type, fn, propagate, once, priority)
        return fn
    return decorator


def listen(
    target: Optional[Type],
    event_type: Union[EventType, str],
    listener: EventListener,
    propagate: bool = True,
    once: bool = False,
    priority: int = 0,
) -> EventRegistration:
    """
    Register an event listener (function form).

    Example:
        def my_listener(target):
            print(f"Inserting {target}")

        registration = listen(User, 'before_insert', my_listener)

        # Later, to remove:
        event.remove(registration)
    """
    return event.listen(target, event_type, listener, propagate, once, priority)


def remove_listener(registration: EventRegistration) -> bool:
    """Remove a previously registered listener."""
    return event.remove(registration)


def dispatch(
    event_type: Union[EventType, str],
    target_instance: Any,
    *args,
    **kwargs,
) -> List[Any]:
    """Dispatch an event synchronously."""
    return event.dispatch(event_type, target_instance, *args, **kwargs)


async def dispatch_async(
    event_type: Union[EventType, str],
    target_instance: Any,
    *args,
    **kwargs,
) -> List[Any]:
    """Dispatch an event asynchronously."""
    return await event.dispatch_async(event_type, target_instance, *args, **kwargs)


# Convenience decorators for common events
def before_insert(target: Type, **kwargs) -> Callable[[EventListener], EventListener]:
    """Decorator for before_insert events."""
    return listens_for(target, EventType.BEFORE_INSERT, **kwargs)


def after_insert(target: Type, **kwargs) -> Callable[[EventListener], EventListener]:
    """Decorator for after_insert events."""
    return listens_for(target, EventType.AFTER_INSERT, **kwargs)


def before_update(target: Type, **kwargs) -> Callable[[EventListener], EventListener]:
    """Decorator for before_update events."""
    return listens_for(target, EventType.BEFORE_UPDATE, **kwargs)


def after_update(target: Type, **kwargs) -> Callable[[EventListener], EventListener]:
    """Decorator for after_update events."""
    return listens_for(target, EventType.AFTER_UPDATE, **kwargs)


def before_delete(target: Type, **kwargs) -> Callable[[EventListener], EventListener]:
    """Decorator for before_delete events."""
    return listens_for(target, EventType.BEFORE_DELETE, **kwargs)


def after_delete(target: Type, **kwargs) -> Callable[[EventListener], EventListener]:
    """Decorator for after_delete events."""
    return listens_for(target, EventType.AFTER_DELETE, **kwargs)


def before_flush(**kwargs) -> Callable[[EventListener], EventListener]:
    """Decorator for before_flush events (global)."""
    return listens_for(None, EventType.BEFORE_FLUSH, **kwargs)


def after_commit(**kwargs) -> Callable[[EventListener], EventListener]:
    """Decorator for after_commit events (global)."""
    return listens_for(None, EventType.AFTER_COMMIT, **kwargs)


class AttributeEvents:
    """
    Mixin class to enable attribute change events on a model.

    Example:
        class User(Table, AttributeEvents):
            name: str
            email: str

        @listens_for(User, 'attribute_set')
        def on_change(target, key, old_value, new_value):
            print(f"{key} changed from {old_value} to {new_value}")
    """

    _tracking_enabled: bool = False
    _original_values: Dict[str, Any] = {}

    def __setattr__(self, key: str, value: Any) -> None:
        if key.startswith('_'):
            super().__setattr__(key, value)
            return

        # Get old value if tracking is enabled
        old_value = getattr(self, key, None) if self._tracking_enabled else None

        # Set the new value
        super().__setattr__(key, value)

        # Dispatch attribute_set event if value changed
        if self._tracking_enabled and old_value != value:
            event.dispatch(EventType.ATTRIBUTE_SET, self, key, old_value, value)

    def enable_tracking(self) -> None:
        """Enable attribute change tracking."""
        self._tracking_enabled = True
        self._original_values = {}
        # Store current values
        for key in dir(self):
            if not key.startswith('_') and not callable(getattr(self, key)):
                self._original_values[key] = getattr(self, key)

    def disable_tracking(self) -> None:
        """Disable attribute change tracking."""
        self._tracking_enabled = False


__all__ = [
    # Core types
    'EventType',
    'EventRegistration',
    'EventDispatcher',

    # Global dispatcher
    'event',

    # Functions
    'listens_for',
    'listen',
    'remove_listener',
    'dispatch',
    'dispatch_async',

    # Convenience decorators
    'before_insert',
    'after_insert',
    'before_update',
    'after_update',
    'before_delete',
    'after_delete',
    'before_flush',
    'after_commit',

    # Mixin
    'AttributeEvents',
]
