# etcd Client Binding for Python using Rust

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

Actual connection establishment with Etcd's gRPC channel will be done
when you call `EtcdClient.connect()`.

This call returns async context manager, which manages `EtcdCommunicator` instance.

```python
async def main():
    async with etcd.connect() as communicator:
        await communicator.put('testkey'.encode(), 'testvalue'.encode())
        value = await communicator.get('testkey'.encode())
        print(bytes(value).decode())  # testvalue
```


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

## Operating with Etcd lock

Just like `EtcdClient.connect()`, you can easilly use etcd lock by calling `EtcdClient.with_lock(lock_name, timeout=None)`.

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

Adding `timeout` parameter to `EtcdClient.with_lock()` call will add a timeout to lock acquiring process.

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
            timeout=5.0,
        )
    ) as communicator:
        value = await communicator.get('testkey'.encode())
        print('second:', bytes(value).decode())

async with etcd.connect() as communicator:
    await communicator.put('testkey'.encode(), 'testvalue'.encode())
await asyncio.gather(first(), second())  # first: testvalue | second: testvalue
```

Adding `ttl` parameter to `EtcdClient.with_lock()` call will force lock to be released after given seconds.

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

You can watch changes on key with `EtcdCommunicator.watch(key)`.

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

Watching changes on keys with specific prefix can be also done by `EtcdCommunicator.watch_prefix(key_prefix)`.

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

You can run etcd transaction by calling `EtcdCommunicator.txn_compare(compares, txn_builder)`.

### Constructing compares

Constructing compare operations can be done by comparing `Compare` instance.

```python
from etcd_client import Compare, CompareOp
compares = [
    Compare.value('cmpkey1', CompareOp.EQUAL, 'foo'),
    Compare.value('cmpkey2', CompareOp.GREATER, 'bar'),
]
```

### Executing transaction calls

```python
async with etcd.connect() as communicator:
    await communicator.put('cmpkey1'.encode(), 'foo'.encode())
    await communicator.put('cmpkey2'.encode(), 'baz'.encode())
    await communicator.put('successkey'.encode(), 'asdf'.encode())

    compares = [
        Compare.value('cmpkey1', CompareOp.EQUAL, 'foo'),
        Compare.value('cmpkey2', CompareOp.GREATER, 'bar'),
    ]

    txn = Txn()
    res = await communicator.txn(txn.when(compares).and_then([TxnOp.get('successkey'.encode())]))
    print(res) # TODO: Need to write response type bindings.
```

## How to build

### Prerequisite

* The Rust development environment (the 2021 edition or later) using [`rustup`](https://rustup.rs/) or your package manager
* The Python development environment (3.10 or later) using [`pyenv`](https://github.com/pyenv/pyenv#installation) or your package manager

### Build instruction

First, create a virtualenv (either using the standard venv package, pyenv, or
whatever your favorite).  Then, install the PEP-517 build toolchain and run it.

```shell
pip install -U pip build setuptools
python -m build --sdist --wheel
```

It will automatically install build dependencies like
[`maturin`](https://github.com/PyO3/maturin) and build the wheel and source
distributions under the `dist/` directory.

## How to develop and test

(TODO: run maturin for an editable setup)
