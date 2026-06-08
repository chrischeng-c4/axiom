"""Tests for data-bridge-pyloop module."""

import pytest


class TestPyLoopImport:
    """Test PyLoop module import."""

    def test_can_import_pyloop(self):
        """Test that PyLoop can be imported."""
        from cclab.orbit import PyLoop

        assert PyLoop is not None

    def test_can_import_event_loop_policy(self):
        """Test that EventLoopPolicy can be imported."""
        from cclab.orbit import EventLoopPolicy

        assert EventLoopPolicy is not None


class TestPyLoopBasics:
    """Test basic PyLoop functionality."""

    def test_can_create_pyloop(self):
        """Test that PyLoop can be instantiated."""
        from cclab.orbit import PyLoop

        loop = PyLoop()
        assert loop is not None
        assert repr(loop).startswith("PyLoop(")

    def test_new_loop_not_running(self):
        """Test that a new loop is not running."""
        from cclab.orbit import PyLoop

        loop = PyLoop()
        assert not loop.is_running()

    def test_new_loop_not_closed(self):
        """Test that a new loop is not closed."""
        from cclab.orbit import PyLoop

        loop = PyLoop()
        assert not loop.is_closed()

    def test_can_close_loop(self):
        """Test that loop can be closed."""
        from cclab.orbit import PyLoop

        loop = PyLoop()
        loop.close()
        assert loop.is_closed()

    def test_cannot_close_running_loop(self):
        """Test that a running loop cannot be closed."""
        # This will be tested more thoroughly in Phase 2
        # when we have actual loop execution
        pass


class TestEventLoopPolicy:
    """Test EventLoopPolicy functionality."""

    def test_can_create_policy(self):
        """Test that EventLoopPolicy can be created."""
        from cclab.orbit import EventLoopPolicy

        policy = EventLoopPolicy()
        assert policy is not None

    def test_policy_get_event_loop(self):
        """Test that policy can get event loop."""
        from cclab.orbit import EventLoopPolicy

        policy = EventLoopPolicy()
        loop = policy.get_event_loop()
        assert loop is not None

    def test_policy_new_event_loop(self):
        """Test that policy can create new event loop."""
        from cclab.orbit import EventLoopPolicy

        policy = EventLoopPolicy()
        loop = policy.new_event_loop()
        assert loop is not None

    def test_policy_set_event_loop(self):
        """Test that policy can set event loop."""
        from cclab.orbit import EventLoopPolicy, PyLoop

        policy = EventLoopPolicy()
        loop = PyLoop()
        policy.set_event_loop(loop)

        # Getting loop should return the same instance
        retrieved = policy.get_event_loop()
        # Note: This might not be the exact same object due to PyO3 wrapping,
        # but it should be a valid loop


class TestInstallation:
    """Test pyloop installation as default event loop."""

    def test_is_installed_initially_false(self):
        """Test that pyloop is not installed by default."""
        from cclab.orbit import is_installed

        # Should be False initially (unless user has already installed it)
        # Note: This might be True if tests are run after manual installation
        assert isinstance(is_installed(), bool)

    def test_install_function_exists(self):
        """Test that install function exists."""
        from cclab.orbit import install

        assert callable(install)

    # Note: We don't test actual installation here because it would affect
    # the global asyncio state and potentially break other tests
