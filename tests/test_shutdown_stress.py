"""
Stress tests for tokio runtime cleanup during Python shutdown.

These tests validate that the automatic graceful shutdown mechanism works correctly
by running multiple subprocess iterations that create and destroy etcd clients.

With automatic cleanup via reference counting, explicit cleanup_runtime() calls
are no longer needed - the runtime is automatically cleaned up when the last
client context exits.

Reference:
    - BA-1976: https://lablup.atlassian.net/browse/BA-1976
    - pyo3-async-runtimes#40: https://github.com/PyO3/pyo3-async-runtimes/issues/40
"""

import os
import subprocess
import sys
import tempfile
from pathlib import Path

import pytest

# Subprocess timeout values (seconds)
DEFAULT_TIMEOUT = 10
THREADED_TIMEOUT = 20  # Multi-threaded tests need more time
MIXED_CONCURRENCY_TIMEOUT = 30  # Most complex scenario


def _make_test_script(test_code: str, etcd_port: int) -> str:
    """Create a test script for subprocess execution."""
    return f"""
import asyncio
from tests.harness import AsyncEtcd, ConfigScopes, HostPortPair

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


def _run_subprocess_test(
    script_content: str, iterations: int = 10, timeout: int = DEFAULT_TIMEOUT
) -> None:
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
                timeout=timeout,
                env=env,
            )

            if result.returncode != 0:
                failures.append({
                    "iteration": i + 1,
                    "returncode": result.returncode,
                    "stderr": result.stderr,
                    "stdout": result.stdout,
                })

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
    """Test shutdown with an active watch stream."""
    etcd_port = etcd_container.get_exposed_port(2379)

    test_code = """
    async with etcd:
        watch_iter = etcd.watch("test_key")
        await etcd.put("test_key", "value1")
"""
    script = _make_test_script(test_code, etcd_port)
    _run_subprocess_test(script, iterations=20)


@pytest.mark.asyncio
async def test_shutdown_with_concurrent_operations(etcd_container) -> None:
    """Test shutdown with many concurrent operations."""
    etcd_port = etcd_container.get_exposed_port(2379)

    test_code = """
    async with etcd:
        tasks = []
        for i in range(50):
            tasks.append(etcd.put(f"key_{i}", f"value_{i}"))
        await asyncio.gather(*tasks)
"""
    script = _make_test_script(test_code, etcd_port)
    _run_subprocess_test(script, iterations=20)


@pytest.mark.asyncio
async def test_shutdown_rapid_subprocess(etcd_container) -> None:
    """Test rapid subprocess creation/destruction."""
    etcd_port = etcd_container.get_exposed_port(2379)

    test_code = """
    async with etcd:
        await etcd.put("rapid_test", "value")
"""
    script = _make_test_script(test_code, etcd_port)
    _run_subprocess_test(script, iterations=50)


@pytest.mark.asyncio
async def test_shutdown_sanity_check(etcd_container) -> None:
    """Verify subprocess test infrastructure works correctly."""
    etcd_port = etcd_container.get_exposed_port(2379)

    test_code = """
    async with etcd:
        await etcd.put("sanity", "check")
        value = await etcd.get("sanity")
        assert value == "check"
"""
    script = _make_test_script(test_code, etcd_port)
    _run_subprocess_test(script, iterations=5)


@pytest.mark.asyncio
async def test_shutdown_multi_async_tasks(etcd_container) -> None:
    """Test shutdown with multiple concurrent async tasks sharing one event loop."""
    etcd_port = etcd_container.get_exposed_port(2379)

    script = f"""
import asyncio
from tests.harness import AsyncEtcd, ConfigScopes, HostPortPair

async def worker(worker_id: int):
    etcd = AsyncEtcd(
        addr=HostPortPair(host="127.0.0.1", port={etcd_port}),
        namespace=f"test_multi_task_{{worker_id}}",
        scope_prefix_map={{ConfigScopes.GLOBAL: "global"}},
    )
    async with etcd:
        for i in range(5):
            await etcd.put(f"key_{{worker_id}}_{{i}}", f"value_{{i}}")
            await asyncio.sleep(0.01)

async def main():
    tasks = [asyncio.create_task(worker(i)) for i in range(5)]
    await asyncio.gather(*tasks)

if __name__ == "__main__":
    asyncio.run(main())
"""
    _run_subprocess_test(script, iterations=20)


@pytest.mark.asyncio
async def test_shutdown_multi_threaded(etcd_container) -> None:
    """Test shutdown with multiple threads, each running its own event loop."""
    etcd_port = etcd_container.get_exposed_port(2379)

    script = f"""
import asyncio
import threading
from tests.harness import AsyncEtcd, ConfigScopes, HostPortPair

def thread_worker(thread_id: int, results: list, errors: list):
    try:
        async def async_work():
            etcd = AsyncEtcd(
                addr=HostPortPair(host="127.0.0.1", port={etcd_port}),
                namespace=f"test_multi_thread_{{thread_id}}",
                scope_prefix_map={{ConfigScopes.GLOBAL: "global"}},
            )
            async with etcd:
                for i in range(3):
                    await etcd.put(f"key_{{thread_id}}_{{i}}", f"value_{{i}}")
        asyncio.run(async_work())
        results.append(thread_id)
    except Exception as e:
        errors.append((thread_id, str(e)))

def main():
    results, errors, threads = [], [], []
    for i in range(4):
        t = threading.Thread(target=thread_worker, args=(i, results, errors))
        threads.append(t)
        t.start()
    for t in threads:
        t.join(timeout=10)
    if errors:
        raise RuntimeError(f"Thread errors: {{errors}}")
    if len(results) != 4:
        raise RuntimeError(f"Expected 4 completed threads, got {{len(results)}}")

if __name__ == "__main__":
    main()
"""
    _run_subprocess_test(script, iterations=10, timeout=THREADED_TIMEOUT)


@pytest.mark.asyncio
async def test_shutdown_mixed_concurrency(etcd_container) -> None:
    """Test shutdown with multiple threads, each running multiple async tasks."""
    etcd_port = etcd_container.get_exposed_port(2379)

    script = f"""
import asyncio
import threading
from tests.harness import AsyncEtcd, ConfigScopes, HostPortPair

def thread_worker(thread_id: int, results: list, errors: list):
    try:
        async def async_task(task_id: int):
            etcd = AsyncEtcd(
                addr=HostPortPair(host="127.0.0.1", port={etcd_port}),
                namespace=f"test_mixed_{{thread_id}}_{{task_id}}",
                scope_prefix_map={{ConfigScopes.GLOBAL: "global"}},
            )
            async with etcd:
                await etcd.put("key", f"value_{{thread_id}}_{{task_id}}")

        async def run_tasks():
            tasks = [asyncio.create_task(async_task(i)) for i in range(3)]
            await asyncio.gather(*tasks)

        asyncio.run(run_tasks())
        results.append(thread_id)
    except Exception as e:
        errors.append((thread_id, str(e)))

def main():
    results, errors, threads = [], [], []
    for i in range(3):
        t = threading.Thread(target=thread_worker, args=(i, results, errors))
        threads.append(t)
        t.start()
    for t in threads:
        t.join(timeout=15)
    if errors:
        raise RuntimeError(f"Thread errors: {{errors}}")
    if len(results) != 3:
        raise RuntimeError(f"Expected 3 completed threads, got {{len(results)}}")

if __name__ == "__main__":
    main()
"""
    _run_subprocess_test(script, iterations=10, timeout=MIXED_CONCURRENCY_TIMEOUT)
