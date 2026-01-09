# etcd-client-py

[![PyPI release version](https://badge.fury.io/py/etcd-client-py.svg)](https://pypi.org/project/etcd-client-py/)
![Wheels](https://img.shields.io/pypi/wheel/etcd-client-py.svg)

Python wrapper of [etcd_client](https://github.com/etcdv3/etcd-client) built with [PyO3](https://github.com/PyO3/pyo3).

## Installation

```bash
pip install etcd_client
```

## Basic usage

```python
from etcd_client import EtcdClient
etcd = EtcdClient(['http://127.0.0.1:2379'])
```

Actual connection establishment with Etcd's gRPC channel will be done when you call `EtcdClient.connect()`.

```python
async def main():
    async with etcd.connect() as communicator:
        await communicator.put('testkey'.encode(), 'testvalue'.encode())
        value = await communicator.get('testkey'.encode())
        print(bytes(value).decode())  # testvalue
```

### Working with key prefixes

`EtcdCommunicator.get_prefix(prefix)` returns a list of key-value pairs matching the given prefix.

```python
async def main():
    async with etcd.connect() as communicator:
        await communicator.put('/testdir'.encode(), 'root'.encode())
        await communicator.put('/testdir/1'.encode(), '1'.encode())
        await communicator.put('/testdir/2'.encode(), '2'.encode())
        await communicator.put('/testdir/2/3'.encode(), '3'.encode())

        test_dir = await communicator.get_prefix('/testdir'.encode())

        for resp in test_dir:
            # ['/testdir', 'root']
            # ['/testdir/1', '1']
            # ['/testdir/2', '2']
            # ['/testdir/2/3', '3']
            print([bytes(v).decode() for v in resp])
```

## Automatic runtime cleanup

The tokio runtime is automatically cleaned up when the last client context exits. In most cases, no explicit cleanup is needed:

```python
import asyncio
from etcd_client import EtcdClient

async def main():
    etcd = EtcdClient(['http://127.0.0.1:2379'])
    async with etcd.connect() as communicator:
        await communicator.put('testkey'.encode(), 'testvalue'.encode())
        value = await communicator.get('testkey'.encode())
        print(bytes(value).decode())
    # Runtime automatically cleaned up when context exits

asyncio.run(main())
```

The library uses reference counting to track active client contexts. When the last context exits, the tokio runtime is gracefully shut down, waiting up to 5 seconds for pending tasks to complete. If you create new clients after this, the runtime is automatically re-initialized.

For advanced use cases requiring explicit control, `cleanup_runtime()` is available:

```python
from etcd_client import cleanup_runtime

# Force cleanup at a specific point (usually not needed)
cleanup_runtime()
```

## Operating with Etcd lock

Just like `EtcdClient.connect()`, you can easily use etcd lock by calling `EtcdClient.with_lock(lock_opts)`.

```python
async def first():
    async with etcd.with_lock(
        EtcdLockOption(
            lock_name='foolock'.encode(),
        )
    ) as communicator:
        value = await communicator.get('testkey'.encode())
        print('first:', bytes(value).decode(), end=' | ')

async def second():
    await asyncio.sleep(0.1)
    async with etcd.with_lock(
        EtcdLockOption(
            lock_name='foolock'.encode(),
        )
    ) as communicator:
        value = await communicator.get('testkey'.encode())
        print('second:', bytes(value).decode())

async with etcd.connect() as communicator:
    await communicator.put('testkey'.encode(), 'testvalue'.encode())
await asyncio.gather(first(), second())  # first: testvalue | second: testvalue
```

### Lock timeout

Adding `timeout` parameter to `EtcdLockOption` will add a timeout to the lock acquiring process.

```python
async def second():
    await asyncio.sleep(0.1)
    async with etcd.with_lock(
        EtcdLockOption(
            lock_name='foolock'.encode(),
            timeout=5.0,
        )
    ) as communicator:
        value = await communicator.get('testkey'.encode())
        print('second:', bytes(value).decode())
```

### Lock TTL

Adding `ttl` parameter to `EtcdLockOption` will force the lock to be released after the given seconds.

```python
async def first():
    async with etcd.with_lock(
        EtcdLockOption(lock_name="foolock".encode(), ttl=5)
    ) as communicator:
        await asyncio.sleep(10)

async def second():
    start = time.time()
    async with etcd.with_lock(
        EtcdLockOption(lock_name="foolock".encode(), ttl=5)
    ) as communicator:
        print(f"acquired lock after {time.time() - start} seconds")

# 'second' acquired lock after 5.247947931289673 seconds
done, _ = await asyncio.wait([
    asyncio.create_task(first()),
    asyncio.create_task(second())
], return_when=asyncio.FIRST_COMPLETED)

for task in done:
    print(task.result())
```

## Watch

You can watch changes on a key with `EtcdCommunicator.watch(key)`.

```python
async def watch():
    async with etcd.connect() as communicator:
        async for event in communicator.watch('testkey'.encode()):
            print(event.event, bytes(event.value).decode())

async def update():
    await asyncio.sleep(0.1)
    async with etcd.connect() as communicator:
        await communicator.put('testkey'.encode(), '1'.encode())
        await communicator.put('testkey'.encode(), '2'.encode())
        await communicator.put('testkey'.encode(), '3'.encode())
        await communicator.put('testkey'.encode(), '4'.encode())
        await communicator.put('testkey'.encode(), '5'.encode())

await asyncio.gather(watch(), update())
# WatchEventType.PUT 1
# WatchEventType.PUT 2
# WatchEventType.PUT 3
# WatchEventType.PUT 4
# WatchEventType.PUT 5
```

### Watch with prefix

Watching changes on keys with a specific prefix can be done with `EtcdCommunicator.watch_prefix(key_prefix)`.

```python
async def watch():
    async with etcd.connect() as communicator:
        async for event in communicator.watch_prefix('/testdir'.encode()):
            print(event.event, bytes(event.key).decode(), bytes(event.value).decode())

async def update():
    await asyncio.sleep(0.1)
    async with etcd.connect() as communicator:
        await communicator.put('/testdir'.encode(), '1'.encode())
        await communicator.put('/testdir/foo'.encode(), '2'.encode())
        await communicator.put('/testdir/bar'.encode(), '3'.encode())
        await communicator.put('/testdir/foo/baz'.encode(), '4'.encode())

await asyncio.gather(watch(), update())
# WatchEventType.PUT /testdir 1
# WatchEventType.PUT /testdir/foo 2
# WatchEventType.PUT /testdir/bar 3
# WatchEventType.PUT /testdir/foo/baz 4
```

## Transaction

You can run etcd transactions by calling `EtcdCommunicator.txn(txn)`.

### Constructing compares

Constructing compare operations can be done using the `Compare` class.

```python
from etcd_client import Compare, CompareOp
compares = [
    Compare.value('cmpkey1'.encode(), CompareOp.EQUAL, 'foo'.encode()),
    Compare.value('cmpkey2'.encode(), CompareOp.GREATER, 'bar'.encode()),
]
```

### Executing transactions

```python
async with etcd.connect() as communicator:
    await communicator.put('cmpkey1'.encode(), 'foo'.encode())
    await communicator.put('cmpkey2'.encode(), 'baz'.encode())
    await communicator.put('successkey'.encode(), 'asdf'.encode())

    compares = [
        Compare.value('cmpkey1'.encode(), CompareOp.EQUAL, 'foo'.encode()),
        Compare.value('cmpkey2'.encode(), CompareOp.GREATER, 'bar'.encode()),
    ]

    res = await communicator.txn(Txn().when(compares).and_then([TxnOp.get('successkey'.encode())]))
    print(res)  # TODO: Need to write response type bindings.
```

## How to build

### Prerequisites

* The Rust development environment (2021 edition or later) using [`rustup`](https://rustup.rs/) or your package manager
* The Python development environment (3.10 or later) using [`pyenv`](https://github.com/pyenv/pyenv#installation) or your package manager

### Build instructions

First, create a virtualenv (using the standard venv package, pyenv, or your preferred tool). Then, install the PEP-517 build toolchain and run it.

```shell
pip install -U pip build setuptools
python -m build --sdist --wheel
```

This will automatically install build dependencies like [`maturin`](https://github.com/PyO3/maturin) and build the wheel and source distributions under the `dist/` directory.

## How to develop and test

### Setup development environment

This project uses [uv](https://docs.astral.sh/uv/) for fast Python package management.

```bash
# Install uv (if not already installed)
curl -LsSf https://astral.sh/uv/install.sh | sh

# Install all dependencies and build the package
make install

# Or manually:
uv sync --all-extras  # Installs all dependencies from pyproject.toml
uv run maturin develop  # Builds and installs the Rust extension
```

### Code quality checks

This project uses ruff for linting/formatting and mypy for type checking:

```bash
make fmt       # Format all code (Python + Rust)
make lint      # Lint all code (Python + Rust)
make fix       # Auto-fix all issues (Python + Rust)
make typecheck # Type check Python code
make check     # Run all checks
```

### Running tests

```bash
make test      # Run tests using uv
uv run pytest  # Or directly with uv

# Tests use testcontainers to automatically spin up etcd
```
