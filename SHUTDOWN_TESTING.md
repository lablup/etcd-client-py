# Tokio Runtime Shutdown Testing and Analysis

## Overview

This document summarizes the investigation into the tokio runtime cleanup issue described in [BA-1976](https://lablup.atlassian.net/browse/BA-1976) and provides improved test cases for reproducing and validating the fix.

## The Problem

### Root Cause

The issue is a race condition during Python interpreter shutdown when background tokio tasks spawned by etcd-client-py are still running. This manifests as:

- Segmentation faults (SIGSEGV)
- `PyGILState_Release` fatal errors
- SIGABRT signals (exit code -6)
- Occurs approximately 50% of the time on fast machines

**Technical Details:**

The current implementation uses `pyo3_async_runtimes::tokio::future_into_py`, which:

1. Stores the tokio runtime in a `OnceCell<Pyo3Runtime>`
2. Only provides reference access (cannot take ownership)
3. Has no explicit cleanup mechanism
4. Leaves orphaned tokio tasks when Python finalization begins
5. These tasks try to call `PyGILState_Release` when GIL state is invalid

See: [pyo3-async-runtimes#40](https://github.com/PyO3/pyo3-async-runtimes/issues/40)

## Comparison with valkey-glide

### Current etcd-client-py Approach

```rust
// In client.rs, communicator.rs, etc.
future_into_py(py, async move {
    // Async operations...
})
```

**Issues:**
- No explicit runtime cleanup
- Runtime stored in static `OnceCell`
- Cannot shutdown runtime (requires ownership)
- Vulnerable to race condition

### valkey-glide Approach

```rust
// Custom runtime management
impl Drop for GlideRt {
    fn drop(&mut self) {
        if let Some(rt) = RUNTIME.get() {
            rt.shutdown_notifier.notify_one();
        }
        if let Some(handle) = self.thread.take() {
            handle.join().expect("GlideRt thread panicked");
        }
    }
}
```

**Advantages:**
- Explicit cleanup via `Drop` trait
- Dedicated runtime thread with shutdown notification
- Graceful thread joining ensures completion
- More robust shutdown handling

**Key Differences:**

| Aspect | etcd-client-py | valkey-glide |
|--------|----------------|--------------|
| Runtime Creation | Implicit via `future_into_py` | Explicit via `GlideRt` |
| Runtime Storage | `OnceCell<Pyo3Runtime>` | Custom `GlideRt` struct |
| Cleanup Mechanism | None (implicit drop) | Explicit `Drop` implementation |
| Shutdown Control | Reference only | Full ownership |
| Thread Management | Shared thread pool | Dedicated runtime thread |
| Shutdown Signaling | None | `shutdown_notifier` |
| Thread Joining | No | Yes (ensures completion) |
| Robustness | Vulnerable | More robust |

## Improved Test Suite

### New Test Files

1. **`tests/test_shutdown_stress.py`** - Comprehensive stress tests
2. **`tests/test_cross_library_shutdown.py`** - Cross-library interaction tests

### Test Scenarios

#### 1. Active Watch Shutdown (`test_shutdown_with_active_watch`)
- **Purpose:** Test shutdown with long-lived background watch tasks
- **Iterations:** 20
- **Why:** Watches create persistent background tokio tasks most likely to hit the race condition

#### 2. Multiple Concurrent Operations (`test_shutdown_with_multiple_concurrent_operations`)
- **Purpose:** Maximize number of in-flight tokio tasks during shutdown
- **Operations:** 50 concurrent put operations
- **Iterations:** 20
- **Why:** More concurrent tasks = higher chance of hitting race condition

#### 3. Rapid Client Creation (`test_shutdown_with_rapid_client_creation`)
- **Purpose:** Stress runtime initialization/cleanup cycle
- **Clients:** 5 sequential client instances
- **Iterations:** 20
- **Why:** Tests runtime creation/destruction paths

#### 4. Combined Stress (`test_shutdown_with_watch_and_concurrent_ops`)
- **Purpose:** Most aggressive test combining multiple stress factors
- **Components:** 3 watch streams + 30 concurrent operations
- **Iterations:** 30
- **Why:** Maximum stress on runtime cleanup

#### 5. Ultra-Rapid Subprocess (`test_shutdown_ultra_rapid_subprocess`)
- **Purpose:** Short-lived processes with minimal operations
- **Iterations:** 50
- **Why:** Highest iteration count to maximize race condition exposure

#### 6. Cross-Library Test (`test_etcd_and_valkey_concurrent_shutdown`)
- **Purpose:** Validate interaction between etcd-client-py and valkey-glide
- **Status:** Infrastructure pending (requires valkey-glide installation + Valkey container)
- **Why:** Ensures both libraries' runtime cleanup doesn't conflict

### Improvements Over Original Test

The original `test_subprocess_segfault_reproduction`:
- Single scenario (basic put operation)
- Only 5 iterations
- No watch streams or concurrent operations
- Lower reproducibility

New test suite:
- 6 different stress scenarios
- 20-50 iterations per scenario
- Tests watch streams, concurrent ops, rapid creation
- Much higher reproducibility
- Better diagnostic output on failure
- Cross-library testing support

## Running the Tests

### Basic Tests

```bash
# Run all shutdown stress tests
uv run pytest tests/test_shutdown_stress.py -v

# Run specific test
uv run pytest tests/test_shutdown_stress.py::test_shutdown_with_active_watch -v

# Run with more verbose output
uv run pytest tests/test_shutdown_stress.py -vv -s
```

### Cross-Library Tests

```bash
# First, install valkey-glide (optional)
uv pip install valkey-glide

# Run cross-library tests
uv run pytest tests/test_cross_library_shutdown.py -v
```

**Note:** The cross-library test with valkey-glide is currently skipped because:
1. Requires `valkey-glide` installation
2. Requires running Valkey/Redis instance
3. Needs Valkey container fixture in `conftest.py`

### Expected Behavior

**Before Fix:**
- Tests should fail intermittently (especially on fast machines)
- Common failure modes:
  - Exit code -11 (SIGSEGV)
  - Exit code -6 (SIGABRT)
  - `PyGILState_Release` errors in stderr
  - Fatal Python errors about thread state

**After Fix:**
- All tests should pass consistently
- All iterations should return exit code 0
- No GIL-related errors in output

## Recommendations

### Short-term Fix (Current)

The current mitigation adds a small sleep delay before exit to allow tokio tasks to complete:

```python
import time
time.sleep(0.1)
```

This is implemented in the backend.ai main command processor.

### Long-term Fix (Recommended)

Implement explicit runtime cleanup similar to valkey-glide:

1. **Create Runtime Wrapper:**
   ```rust
   struct EtcdRt {
       thread: Option<JoinHandle<()>>,
       shutdown_notifier: Arc<Notify>,
   }
   ```

2. **Implement Drop:**
   ```rust
   impl Drop for EtcdRt {
       fn drop(&mut self) {
           if let Some(rt) = RUNTIME.get() {
               rt.shutdown_notifier.notify_one();
           }
           if let Some(handle) = self.thread.take() {
               handle.join().expect("Runtime thread panicked");
           }
       }
   }
   ```

3. **Use Dedicated Runtime Thread:**
   - Create single-threaded tokio runtime
   - Run in dedicated thread
   - Block on shutdown notification
   - Join thread on cleanup

4. **Register Python Cleanup:**
   ```rust
   #[pyfunction]
   fn _cleanup_runtime() {
       // Explicit cleanup function callable from Python
   }
   ```

### Alternative Approaches

1. **Wait for pyo3-async-runtimes fix:**
   - Upstream issue: https://github.com/PyO3/pyo3-async-runtimes/issues/40
   - Proposed: Change `OnceCell<Runtime>` to `Option<Runtime>`
   - Allows taking ownership for shutdown

2. **Use atexit handler:**
   ```python
   import atexit
   from etcd_client import _cleanup_runtime
   atexit.register(_cleanup_runtime)
   ```

3. **Python-side runtime management:**
   - Create Python wrapper that manages lifecycle
   - Ensure cleanup in `__del__` or context manager

## Testing Strategy

### Continuous Integration

Add shutdown stress tests to CI pipeline:

```yaml
- name: Run shutdown stress tests
  run: |
    uv run pytest tests/test_shutdown_stress.py -v
    uv run pytest tests/test_cross_library_shutdown.py -v
```

### Local Development

Before committing changes:

```bash
# Run full test suite including stress tests
make test

# Run stress tests multiple times to ensure stability
for i in {1..5}; do
  echo "Run $i"
  uv run pytest tests/test_shutdown_stress.py -v || break
done
```

### Regression Testing

After implementing fix:

1. Run original test: `uv run pytest tests/test.py::test_subprocess_segfault_reproduction -v`
2. Run new stress tests: `uv run pytest tests/test_shutdown_stress.py -v`
3. Repeat multiple times to ensure consistency
4. Test on different hardware (fast/slow machines)
5. Test on different platforms (Linux/macOS)

## References

- **BA-1976:** https://lablup.atlassian.net/browse/BA-1976
- **pyo3-async-runtimes#40:** https://github.com/PyO3/pyo3-async-runtimes/issues/40
- **valkey-glide:** https://github.com/valkey-io/valkey-glide
- **backend.ai#5290:** https://github.com/lablup/backend.ai/issues/5290
- **lance#4152:** https://github.com/lance-format/lance/issues/4152

## Summary

The new test suite provides:

1. **Better Reproducibility:** 6 scenarios with 20-50 iterations each
2. **Better Coverage:** Tests watches, concurrent ops, rapid creation
3. **Better Diagnostics:** Detailed failure reporting
4. **Cross-Library Testing:** Framework for testing with valkey-glide
5. **Documentation:** Clear explanation of issue and solutions

The comparison with valkey-glide reveals that **explicit runtime cleanup via Drop trait** is the most robust long-term solution. The current `pyo3_async_runtimes` approach is fundamentally limited by its use of `OnceCell` which prevents proper shutdown.
