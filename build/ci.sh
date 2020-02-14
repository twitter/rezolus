#!/bin/bash -ev

## Install toolchain
if [[ "${TRAVIS_DIST}" == "xenial" ]]; then
	if ${LLVM_VERSION} = "8"; then
		wget -O - https://apt.llvm.org/llvm-snapshot.gpg.key|sudo apt-key add -
        echo "deb http://apt.llvm.org/xenial/ llvm-toolchain-xenial-8 main" | sudo tee -a /etc/apt/sources.list
        sudo add-apt-repository --yes ppa:ubuntu-toolchain-r/ppa
        sudo apt-get update
        sudo apt-get install linux-headers-`uname -r`
        sudo apt-get --yes install bison build-essential cmake flex git libclang-common-8-dev libelf-dev libllvm8 libz-dev lldb-8 llvm-8 llvm-8-dev llvm-8-runtime
	fi
	if [[ "${LLVM_VERSION}" == "9" ]]; then
		wget -O - https://apt.llvm.org/llvm-snapshot.gpg.key|sudo apt-key add -
        echo "deb http://apt.llvm.org/xenial/ llvm-toolchain-xenial-9 main" | sudo tee -a /etc/apt/sources.list
        sudo add-apt-repository --yes ppa:ubuntu-toolchain-r/ppa
        sudo apt-get update
        sudo apt-get install linux-headers-`uname -r`
        sudo apt-get --yes install bison build-essential cmake flex git libclang-common-9-dev libelf-dev libllvm9 libz-dev lldb-9 llvm-9 llvm-9-dev llvm-9-runtime
	fi
fi
if [[ "${TRAVIS_DIST}" == "bionic" ]]; then
	if [[ "${LLVM_VERSION}" == "10" ]]; then
		wget -O - https://apt.llvm.org/llvm-snapshot.gpg.key|sudo apt-key add -
        echo "deb http://apt.llvm.org/bionic/ llvm-toolchain-bionic-10 main" | sudo tee -a /etc/apt/sources.list
        sudo add-apt-repository --yes ppa:ubuntu-toolchain-r/ppa
        sudo apt-get update
        sudo apt-get install linux-headers-`uname -r`
        sudo apt-get --yes install bison build-essential cmake flex libfl-dev git libclang-common-10-dev libelf-dev libllvm10 libz-dev lldb-10 llvm-10 llvm-10-dev llvm-10-runtime
	fi
fi

## Optionally build/install BCC
if [ ! -z "${BCC_VERSION}" ]; then
	git clone https://github.com/iovisor/bcc || true
    cd bcc
    git checkout master
    git pull
    if [[ "${BCC_VERSION}" == "0.10.0" ]]; then
    	git checkout remotes/origin/tag_v0.10.0
    fi
    if [[ "${BCC_VERSION}" == "0.11.0" ]]; then
    	git checkout 0fa419a64e71984d42f107c210d3d3f0cc82d59a
    fi
    if [[ "${BCC_VERSION}" == "0.12.0" ]]; then
    	git checkout 368a5b0714961953f3e3f61607fa16cb71449c1b
    fi
    if [[ "${BCC_VERSION}" == "latest" ]]; then
    	git checkout 368a5b0714961953f3e3f61607fa16cb71449c1b
    fi
    mkdir -p _build
    cd _build
    cmake .. -DCMAKE_INSTALL_PREFIX=/usr
    make
    sudo make install
    cd ../..
fi

## Build and test
if [ -z "${FEATURES}" ]; then
    cargo build
    cargo test
    cargo build --release
    cargo test --release
    sudo timeout --signal 15 --preserve-status 5.0m target/release/rezolus --config configs/example.toml
    sudo timeout --signal 15 --preserve-status 5.0m target/release/rezolus --config configs/ci.toml
else
	cargo build --features bpf_v0_10_0
	cargo test --features bpf_v0_10_0
	cargo build --release --features bpf_v0_10_0
	cargo test --release --features bpf_v0_10_0
	cargo run --release --features bpf_v0_10_0 -- --version
	sudo timeout --signal 15 --preserve-status 5.0m target/release/rezolus --config configs/example.toml
	sudo timeout --signal 15 --preserve-status 5.0m target/release/rezolus --config configs/ci.toml
fi