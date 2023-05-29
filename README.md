# etcd Client Binding for Python using Rust

[![PyPI release version](https://badge.fury.io/py/etcd-client-py.svg)](https://pypi.org/project/etcd-client-py/)
![Wheels](https://img.shields.io/pypi/wheel/etcd-client-py.svg)

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
