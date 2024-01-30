build:
	maturin build

install:
	maturin build
	pip install .

etcd-clear:
	etcdctl del "" --from-key=true

fmt:
	cargo fmt

lint:
	cargo clippy 
