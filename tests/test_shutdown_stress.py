"""
Stress tests for tokio runtime cleanup during Python shutdown.

This test suite aims to reproduce the segfault issue described in BA-1976
by creating various stress scenarios that maximize the likelihood of hitting
the race condition between Python interpreter shutdown and tokio background tasks.

Reference: https://github.com/PyO3/pyo3-async-runtimes/issues/40
"""

import asyncio
import os
import subprocess
import sys
import tempfile
from pathlib import Path

import pytest


def _make_test_script(test_code: str, etcd_port: int) -> str:
    """Create a temporary Python script for subprocess testing."""
    return f"""
import asyncio
import atexit
import sys

from etcd_client import _cleanup_runtime
from tests.harness import AsyncEtcd, ConfigScopes, HostPortPair

# Register explicit runtime cleanup to wait for tokio shutdown
atexit.register(_cleanup_runtime)

async def main():
    etcd = AsyncEtcd(
        addr=HostPortPair(host="127.0.0.1", port={etcd_port}),
        namespace="test_shutdown_stress",
        scope_prefix_map={{
            ConfigScopes.GLOBAL: "global",
        }},
    )

    {test_code}

if __name__ == "__main__":
    asyncio.run(main())
"""


def _run_subprocess_test(script_content: str, iterations: int = 10) -> None:
    """Run a test script in subprocess multiple times to detect shutdown issues."""
    project_root = str(Path(__file__).parent.parent.resolve())
    env = os.environ.copy()
    env["PYTHONPATH"] = project_root

    with tempfile.NamedTemporaryFile(mode="w", suffix=".py", delete=False) as f:
        f.write(script_content)
        script_path = f.name

    try:
        failures = []
        for i in range(iterations):
            result = subprocess.run(
                [sys.executable, "-u", script_path],
                capture_output=True,
                text=True,
                timeout=10,
                env=env,
            )

            if result.returncode != 0:
                failures.append(
                    {
                        "iteration": i + 1,
                        "returncode": result.returncode,
                        "stderr": result.stderr,
                        "stdout": result.stdout,
                    }
                )

        if failures:
            error_msg = f"Failed {len(failures)}/{iterations} iterations:\n"
            for failure in failures:
                error_msg += (
                    f"\n--- Iteration {failure['iteration']} "
                    f"(exit code {failure['returncode']}) ---\n"
                )
                error_msg += f"stdout: {failure['stdout']}\n"
                error_msg += f"stderr: {failure['stderr']}\n"
            pytest.fail(error_msg)

    finally:
        os.unlink(script_path)


@pytest.mark.asyncio
async def test_shutdown_with_active_watch(etcd_container) -> None:
    """
    Test shutdown with an active watch stream that keeps background tasks alive.

    This scenario is particularly prone to the race condition because:
    - Watch creates long-lived background tokio tasks
    - The task may still be running when Python shutdown begins
    - The tokio task will try to access GIL state during cleanup
    """
    etcd_port = etcd_container.get_exposed_port(2379)

    test_code = """
    async with etcd:
        # Start a watch that will keep a background task alive
        watch_iter = etcd.watch("test_key")

        # Do a quick operation while watch is active
        await etcd.put("test_key", "value1")

        # Exit WITHOUT properly cleaning up the watch
        # This simulates a crash or sudden termination scenario
"""

    script = _make_test_script(test_code, etcd_port)
    _run_subprocess_test(script, iterations=20)


@pytest.mark.asyncio
async def test_shutdown_with_multiple_concurrent_operations(etcd_container) -> None:
    """
    Test shutdown with many concurrent operations in flight.

    This maximizes the number of tokio tasks that might still be running
    during Python shutdown, increasing the likelihood of hitting the race condition.
    """
    etcd_port = etcd_container.get_exposed_port(2379)

    test_code = """
    async with etcd:
        # Create many concurrent operations
        tasks = []
        for i in range(50):
            tasks.append(etcd.put(f"key_{i}", f"value_{i}"))

        # Start them all at once
        await asyncio.gather(*tasks)

        # Immediately exit without giving tasks time to fully clean up
"""

    script = _make_test_script(test_code, etcd_port)
    _run_subprocess_test(script, iterations=20)


@pytest.mark.asyncio
async def test_shutdown_with_rapid_client_creation(etcd_container) -> None:
    """
    Test rapid client creation and destruction in subprocess.

    This stresses the runtime initialization/cleanup cycle by creating
    multiple client instances in quick succession before exiting.
    """
    etcd_port = etcd_container.get_exposed_port(2379)

    test_code = """
    # Create and use multiple clients rapidly
    for i in range(5):
        etcd_temp = AsyncEtcd(
            addr=HostPortPair(host="127.0.0.1", port={port}),
            namespace=f"test_rapid_{{i}}",
            scope_prefix_map={{
                ConfigScopes.GLOBAL: "global",
            }},
        )
        async with etcd_temp:
            await etcd_temp.put(f"key_{{i}}", f"value_{{i}}")

    # Exit immediately after rapid creation/destruction
""".format(
        port=etcd_port
    )

    # Note: This test doesn't use the standard test_code pattern
    # because it needs multiple client instances
    script = f"""
import asyncio
import atexit
import sys

from etcd_client import _cleanup_runtime
from tests.harness import AsyncEtcd, ConfigScopes, HostPortPair

# Register explicit runtime cleanup to wait for tokio shutdown
atexit.register(_cleanup_runtime)

async def main():
    {test_code}

if __name__ == "__main__":
    asyncio.run(main())
"""

    _run_subprocess_test(script, iterations=20)


@pytest.mark.asyncio
async def test_shutdown_with_watch_and_concurrent_ops(etcd_container) -> None:
    """
    Combined stress test: watches + concurrent operations.

    This is the most aggressive test, combining multiple stress factors:
    - Active watch streams with background tasks
    - Many concurrent put/get operations
    - Rapid subprocess termination
    """
    etcd_port = etcd_container.get_exposed_port(2379)

    test_code = """
    async with etcd:
        # Start multiple watch streams
        watch1 = etcd.watch("watch_key1")
        watch2 = etcd.watch("watch_key2")
        watch3 = etcd.watch_prefix("watch_prefix")

        # Create concurrent operations
        ops = []
        for i in range(30):
            ops.append(etcd.put(f"key_{i}", f"value_{i}"))
            if i % 3 == 0:
                ops.append(etcd.get_prefix("key_"))

        # Execute some operations
        await asyncio.gather(*ops[:10])

        # Trigger watch events
        await etcd.put("watch_key1", "event1")
        await etcd.put("watch_key2", "event2")
        await etcd.put("watch_prefix/child", "event3")

        # Exit abruptly with watches still active and operations in flight
"""

    script = _make_test_script(test_code, etcd_port)
    _run_subprocess_test(script, iterations=30)


@pytest.mark.asyncio
async def test_shutdown_ultra_rapid_subprocess(etcd_container) -> None:
    """
    Ultra-rapid subprocess creation/destruction test.

    This test creates many short-lived subprocesses that perform minimal
    operations and exit immediately, maximizing the stress on runtime
    initialization and cleanup paths.
    """
    etcd_port = etcd_container.get_exposed_port(2379)

    test_code = """
    async with etcd:
        # Minimal operation - just connect and write one key
        await etcd.put("rapid_test", "value")
        # Exit immediately
"""

    script = _make_test_script(test_code, etcd_port)

    # Run MANY iterations rapidly to increase chance of hitting race condition
    _run_subprocess_test(script, iterations=50)


@pytest.mark.asyncio
@pytest.mark.skipif(
    "valkey_glide" not in sys.modules,
    reason="valkey-glide not installed - install with: pip install valkey-glide",
)
async def test_shutdown_with_both_etcd_and_valkey(etcd_container) -> None:
    """
    Test shutdown with both etcd-client-py and valkey-glide active.

    This tests the interaction between two Rust libraries that both
    manage tokio runtimes, which could reveal additional race conditions
    or conflicts in runtime cleanup.

    This test requires valkey-glide to be installed:
        pip install valkey-glide
    """
    pytest.skip(
        "This test requires both etcd and valkey infrastructure. "
        "Implement when you have both available in CI/test environment."
    )


# Helper test to verify the test infrastructure works correctly
@pytest.mark.asyncio
async def test_subprocess_infrastructure_sanity_check(etcd_container) -> None:
    """Verify that the subprocess test infrastructure works correctly."""
    etcd_port = etcd_container.get_exposed_port(2379)

    # This should always succeed
    test_code = """
    async with etcd:
        await etcd.put("sanity", "check")
        value = await etcd.get("sanity")
        assert value == "check"
"""

    script = _make_test_script(test_code, etcd_port)
    _run_subprocess_test(script, iterations=5)
