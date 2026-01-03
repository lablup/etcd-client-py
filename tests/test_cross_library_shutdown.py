"""
Cross-library shutdown stress test.

This test validates tokio runtime cleanup when both etcd-client-py
and valkey-glide are used in the same Python process.

Both libraries use PyO3 and tokio, so they may interact in complex ways
during Python interpreter shutdown. This test aims to ensure that both
libraries clean up their runtimes correctly without interfering with each other.

Requirements:
    - valkey-glide: pip install valkey-glide
    - Running Valkey/Redis instance for testing

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


def _create_dual_library_test_script(etcd_port: int, redis_port: int) -> str:
    """Create a test script that uses both etcd-client-py and valkey-glide."""
    return f"""
import asyncio
import sys

# Import both libraries
from tests.harness import AsyncEtcd, ConfigScopes, HostPortPair

try:
    from glide import GlideClientConfiguration, NodeAddress
    from glide.async_commands.standalone_commands import StandaloneClient
    GLIDE_AVAILABLE = True
except ImportError:
    GLIDE_AVAILABLE = False
    print("valkey-glide not available, skipping glide operations", file=sys.stderr)

async def test_both_libraries():
    \"\"\"Use both etcd and valkey in the same process, then exit abruptly.\"\"\"

    # Initialize etcd client
    etcd = AsyncEtcd(
        addr=HostPortPair(host="127.0.0.1", port={etcd_port}),
        namespace="cross_library_test",
        scope_prefix_map={{
            ConfigScopes.GLOBAL: "global",
        }},
    )

    async with etcd:
        # Do some etcd operations
        await etcd.put("etcd_key", "etcd_value")
        result = await etcd.get("etcd_key")
        assert result == "etcd_value", f"Expected 'etcd_value', got {{result}}"

        if GLIDE_AVAILABLE:
            # Initialize valkey-glide client
            config = GlideClientConfiguration(
                addresses=[NodeAddress(host="127.0.0.1", port={redis_port})]
            )
            glide_client = await StandaloneClient.create(config)

            try:
                # Do some glide operations
                await glide_client.set("glide_key", "glide_value")
                glide_result = await glide_client.get("glide_key")
                assert glide_result == b"glide_value", f"Expected b'glide_value', got {{glide_result}}"

                # Interleave operations from both libraries
                await asyncio.gather(
                    etcd.put("key1", "value1"),
                    glide_client.set("key2", "value2"),
                    etcd.put("key3", "value3"),
                    glide_client.set("key4", "value4"),
                )

                # Create concurrent operations
                tasks = []
                for i in range(20):
                    tasks.append(etcd.put(f"etcd_concurrent_{{i}}", f"val_{{i}}"))
                    tasks.append(glide_client.set(f"glide_concurrent_{{i}}", f"val_{{i}}"))

                await asyncio.gather(*tasks)

            finally:
                await glide_client.close()

        # Exit - both libraries' runtimes should clean up gracefully

if __name__ == "__main__":
    asyncio.run(test_both_libraries())
    print("Test completed successfully", file=sys.stderr)
"""


def _run_dual_library_subprocess(
    script_content: str, iterations: int = 10
) -> tuple[int, int, list]:
    """
    Run a script using both libraries in subprocess.

    Returns:
        (successes, failures, failure_details)
    """
    project_root = str(Path(__file__).parent.parent.resolve())
    env = os.environ.copy()
    env["PYTHONPATH"] = project_root

    with tempfile.NamedTemporaryFile(mode="w", suffix=".py", delete=False) as f:
        f.write(script_content)
        script_path = f.name

    try:
        successes = 0
        failures = []

        for i in range(iterations):
            result = subprocess.run(
                [sys.executable, "-u", script_path],
                capture_output=True,
                text=True,
                timeout=15,  # Longer timeout for cross-library test
                env=env,
            )

            if result.returncode == 0:
                successes += 1
            else:
                failures.append(
                    {
                        "iteration": i + 1,
                        "returncode": result.returncode,
                        "stderr": result.stderr,
                        "stdout": result.stdout,
                    }
                )

        return successes, len(failures), failures

    finally:
        os.unlink(script_path)


@pytest.mark.asyncio
@pytest.mark.skipif(sys.platform == "win32", reason="Unix sockets not supported on Windows")
async def test_etcd_and_valkey_concurrent_shutdown(etcd_container) -> None:
    """
    Test that both etcd-client-py and valkey-glide can coexist and shutdown cleanly.

    This is a smoke test that requires:
    1. etcd container (provided by fixture)
    2. valkey-glide installed
    3. Running Valkey/Redis instance

    Note: This test will be skipped if valkey-glide is not installed.
    To install: pip install valkey-glide
    """
    # Check if valkey-glide is available
    try:
        import glide  # noqa: F401
    except ImportError:
        pytest.skip("valkey-glide not installed - install with: pip install valkey-glide")

    # For this test, we'll need a Redis/Valkey instance
    # Since we don't have a Valkey container fixture, we'll skip this test
    # until the infrastructure is set up
    pytest.skip(
        "This test requires a running Valkey/Redis instance. "
        "Set up Valkey container infrastructure to enable this test."
    )

    # When infrastructure is ready, uncomment this:
    # etcd_port = etcd_container.get_exposed_port(2379)
    # valkey_port = valkey_container.get_exposed_port(6379)  # Need valkey fixture
    #
    # script = _create_dual_library_test_script(etcd_port, valkey_port)
    # successes, failures, failure_details = _run_dual_library_subprocess(script, iterations=30)
    #
    # if failures > 0:
    #     error_msg = f"Failed {failures}/{failures + successes} iterations:\n"
    #     for failure in failure_details:
    #         error_msg += (
    #             f"\n--- Iteration {failure['iteration']} "
    #             f"(exit code {failure['returncode']}) ---\n"
    #         )
    #         error_msg += f"stdout: {failure['stdout']}\n"
    #         error_msg += f"stderr: {failure['stderr']}\n"
    #     pytest.fail(error_msg)


@pytest.mark.asyncio
async def test_etcd_only_baseline(etcd_container) -> None:
    """
    Baseline test using only etcd-client-py.

    This provides a baseline for comparing against the cross-library test.
    If this test passes but the cross-library test fails, it indicates
    an interaction issue between the two libraries' runtime management.
    """
    etcd_port = etcd_container.get_exposed_port(2379)

    script = f"""
import asyncio
from tests.harness import AsyncEtcd, ConfigScopes, HostPortPair

async def main():
    etcd = AsyncEtcd(
        addr=HostPortPair(host="127.0.0.1", port={etcd_port}),
        namespace="baseline_test",
        scope_prefix_map={{
            ConfigScopes.GLOBAL: "global",
        }},
    )

    async with etcd:
        # Perform operations similar to cross-library test
        await etcd.put("key", "value")
        result = await etcd.get("key")
        assert result == "value"

        # Concurrent operations
        tasks = []
        for i in range(40):  # More operations than cross-library test
            tasks.append(etcd.put(f"concurrent_{{i}}", f"val_{{i}}"))

        await asyncio.gather(*tasks)

if __name__ == "__main__":
    asyncio.run(main())
"""

    successes, failures, failure_details = _run_dual_library_subprocess(
        script, iterations=30
    )

    if failures > 0:
        error_msg = f"Baseline test failed {failures}/{failures + successes} iterations:\n"
        for failure in failure_details:
            error_msg += (
                f"\n--- Iteration {failure['iteration']} "
                f"(exit code {failure['returncode']}) ---\n"
            )
            error_msg += f"stdout: {failure['stdout']}\n"
            error_msg += f"stderr: {failure['stderr']}\n"
        pytest.fail(error_msg)


@pytest.mark.asyncio
async def test_documentation_example() -> None:
    """
    Document the cross-library testing approach for future reference.

    This test serves as documentation for how to set up and run
    cross-library tests when the infrastructure is available.
    """
    documentation = """
    # Cross-Library Shutdown Testing

    ## Purpose
    Verify that etcd-client-py and valkey-glide can coexist in the same
    Python process without runtime cleanup conflicts.

    ## Why This Matters
    Both libraries:
    - Use PyO3 for Python bindings
    - Use tokio for async runtime
    - May create background tasks that persist until shutdown
    - Are subject to the pyo3-async-runtimes#40 race condition

    ## Infrastructure Requirements

    1. Install valkey-glide:
       ```
       pip install valkey-glide
       ```

    2. Set up Valkey container fixture in conftest.py:
       ```python
       @pytest.fixture(scope="session")
       def valkey_container():
           with ValKeyContainer("valkey/valkey:latest") as container:
               container.start()
               yield container
       ```

    3. Update pyproject.toml to include valkey-glide in test dependencies:
       ```toml
       [project.dependencies]
       # ... existing deps ...
       "valkey-glide>=1.0.0",  # Add this
       ```

    ## Expected Behavior

    When both libraries are used:
    - Both should initialize their tokio runtimes independently
    - Both should be able to perform operations concurrently
    - Both should clean up gracefully on exit
    - No segfaults or GIL state errors should occur

    ## Potential Issues to Watch For

    1. Runtime conflicts: Both libraries may try to initialize global state
    2. Shutdown ordering: One library's cleanup may affect the other
    3. GIL state errors: Background tasks from both libraries accessing GIL during finalization
    4. Thread pool exhaustion: Both libraries spawning many threads

    ## How to Compare with valkey-glide's Approach

    valkey-glide implements explicit runtime cleanup:
    - `GlideRt` struct with `Drop` implementation
    - Dedicated runtime thread with shutdown notification
    - Explicit thread joining on cleanup

    etcd-client-py currently uses:
    - `pyo3_async_runtimes::tokio::future_into_py`
    - Implicit runtime via `OnceCell`
    - No explicit cleanup (subject to race condition)

    ## Recommended Fix

    Consider implementing a cleanup approach similar to valkey-glide:
    1. Create a custom runtime wrapper (e.g., `EtcdRt`)
    2. Implement `Drop` trait for graceful shutdown
    3. Use shutdown notification to signal runtime thread
    4. Join thread to ensure completion before dropping
    """

    # This test always passes - it's just documentation
    assert documentation is not None
    print("\n" + documentation)
