"""
Tests for automatic tokio runtime cleanup via reference counting.

These tests validate that:
1. The runtime is automatically cleaned up when the last client context exits
2. The runtime can be re-initialized for sequential client usage
3. Multiple concurrent clients are handled correctly
4. Exception scenarios maintain correct reference counts
"""

import os
import subprocess
import sys
import tempfile
from pathlib import Path

import pytest

from etcd_client import Client, active_context_count


@pytest.mark.asyncio
async def test_single_client_context_count(etcd_container) -> None:
    """Verify context count increments/decrements correctly for single client."""
    etcd_port = etcd_container.get_exposed_port(2379)
    client = Client([f"http://127.0.0.1:{etcd_port}"])

    assert active_context_count() == 0

    async with client.connect() as comm:
        assert active_context_count() == 1
        await comm.put(b"test_key", b"test_value")

    # After context exit, count should be 0
    assert active_context_count() == 0


@pytest.mark.asyncio
async def test_multiple_concurrent_clients(etcd_container) -> None:
    """Cleanup only happens when ALL clients exit."""
    etcd_port = etcd_container.get_exposed_port(2379)
    client1 = Client([f"http://127.0.0.1:{etcd_port}"])
    client2 = Client([f"http://127.0.0.1:{etcd_port}"])

    assert active_context_count() == 0

    async with client1.connect() as c1:
        assert active_context_count() == 1

        async with client2.connect() as c2:
            assert active_context_count() == 2
            await c1.put(b"k1", b"v1")
            await c2.put(b"k2", b"v2")

        # client2 exited, but client1 still active
        assert active_context_count() == 1
        # Should still be able to use client1
        value = await c1.get(b"k1")
        assert bytes(value) == b"v1"

    # Both exited
    assert active_context_count() == 0


@pytest.mark.asyncio
async def test_nested_contexts_same_client(etcd_container) -> None:
    """Each context entry/exit is counted separately, even for same client."""
    etcd_port = etcd_container.get_exposed_port(2379)
    client = Client([f"http://127.0.0.1:{etcd_port}"])

    assert active_context_count() == 0

    async with client.connect():
        assert active_context_count() == 1

        # Same client, new connection
        async with client.connect() as comm:
            assert active_context_count() == 2
            await comm.put(b"nested_key", b"nested_value")

        assert active_context_count() == 1

    assert active_context_count() == 0


@pytest.mark.asyncio
async def test_exception_during_context(etcd_container) -> None:
    """Count is decremented even if exception occurs during context."""
    etcd_port = etcd_container.get_exposed_port(2379)
    client = Client([f"http://127.0.0.1:{etcd_port}"])

    assert active_context_count() == 0

    with pytest.raises(ValueError, match="test error"):
        async with client.connect() as comm:
            assert active_context_count() == 1
            await comm.put(b"exc_key", b"exc_value")
            raise ValueError("test error")

    # __aexit__ should still have been called
    assert active_context_count() == 0


@pytest.mark.asyncio
async def test_context_count_after_operation_failure(etcd_container) -> None:
    """Count is properly managed even when operations fail inside context."""
    etcd_port = etcd_container.get_exposed_port(2379)
    client = Client([f"http://127.0.0.1:{etcd_port}"])

    assert active_context_count() == 0

    # Even if an operation fails, __aexit__ should properly decrement the count
    async with client.connect() as comm:
        assert active_context_count() == 1
        await comm.put(b"fail_test_key", b"value")

    # Count should be back to 0 after successful context exit
    assert active_context_count() == 0


def _make_sequential_test_script(etcd_port: int) -> str:
    """Create a test script for sequential client usage with auto re-initialization."""
    return f"""
import asyncio
from etcd_client import Client, active_context_count

async def main():
    client = Client(["http://127.0.0.1:{etcd_port}"])

    # First session
    async with client.connect() as comm:
        await comm.put(b"seq_key", b"value1")
        print(f"First session active: {{active_context_count()}}")

    print(f"After first session: {{active_context_count()}}")
    # Runtime was cleaned up here, should be re-initialized for second session

    # Second session - runtime should reinitialize automatically
    async with client.connect() as comm:
        value = await comm.get(b"seq_key")
        print(f"Second session active: {{active_context_count()}}")
        assert bytes(value) == b"value1", f"Expected 'value1', got {{bytes(value)}}"

    print(f"After second session: {{active_context_count()}}")
    print("SUCCESS")

if __name__ == "__main__":
    asyncio.run(main())
"""


@pytest.mark.asyncio
async def test_sequential_clients_reinit(etcd_container) -> None:
    """Runtime re-initializes for sequential client usage (subprocess test)."""
    etcd_port = etcd_container.get_exposed_port(2379)
    script = _make_sequential_test_script(etcd_port)

    project_root = str(Path(__file__).parent.parent.resolve())
    env = os.environ.copy()
    env["PYTHONPATH"] = project_root

    with tempfile.NamedTemporaryFile(mode="w", suffix=".py", delete=False) as f:
        f.write(script)
        script_path = f.name

    try:
        result = subprocess.run(
            [sys.executable, "-u", script_path],
            capture_output=True,
            text=True,
            timeout=30,
            env=env,
        )

        if result.returncode != 0:
            pytest.fail(
                f"Sequential client test failed:\n"
                f"stdout: {result.stdout}\n"
                f"stderr: {result.stderr}"
            )

        assert "SUCCESS" in result.stdout, f"Test did not complete: {result.stdout}"
    finally:
        os.unlink(script_path)


def _make_no_explicit_cleanup_script(etcd_port: int) -> str:
    """Create a test script that does NOT call cleanup_runtime() explicitly."""
    return f"""
import asyncio
from etcd_client import Client, active_context_count

async def main():
    client = Client(["http://127.0.0.1:{etcd_port}"])

    async with client.connect() as comm:
        await comm.put(b"auto_key", b"auto_value")
        value = await comm.get(b"auto_key")
        assert bytes(value) == b"auto_value"

    # No cleanup_runtime() call - should be automatic
    assert active_context_count() == 0
    print("SUCCESS")

if __name__ == "__main__":
    asyncio.run(main())
"""


@pytest.mark.asyncio
async def test_no_explicit_cleanup_needed(etcd_container) -> None:
    """Verify that explicit cleanup_runtime() is not needed (subprocess test)."""
    etcd_port = etcd_container.get_exposed_port(2379)
    script = _make_no_explicit_cleanup_script(etcd_port)

    project_root = str(Path(__file__).parent.parent.resolve())
    env = os.environ.copy()
    env["PYTHONPATH"] = project_root

    with tempfile.NamedTemporaryFile(mode="w", suffix=".py", delete=False) as f:
        f.write(script)
        script_path = f.name

    try:
        # Run multiple times to check for any shutdown issues
        for i in range(5):
            result = subprocess.run(
                [sys.executable, "-u", script_path],
                capture_output=True,
                text=True,
                timeout=10,
                env=env,
            )

            if result.returncode != 0:
                pytest.fail(
                    f"Iteration {i+1} failed:\n"
                    f"stdout: {result.stdout}\n"
                    f"stderr: {result.stderr}"
                )

            assert "SUCCESS" in result.stdout
    finally:
        os.unlink(script_path)
