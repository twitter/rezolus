#!/bin/bash -ev

## Add LLVM repo
wget -O - https://apt.llvm.org/llvm-snapshot.gpg.key|sudo apt-key add -
echo "deb http://apt.llvm.org/${TRAVIS_DIST}/ llvm-toolchain-${TRAVIS_DIST}-${LLVM_VERSION} main" | sudo tee -a /etc/apt/sources.list
sudo add-apt-repository --yes ppa:ubuntu-toolchain-r/ppa

## Update apt
sudo apt-get update

## Install kernel headers for matching version
sudo apt-get install linux-headers-"$(uname -r)"

## Dist specific dependencies
if [[ "${TRAVIS_DIST}" == "xenial" ]]; then
    sudo apt-get --yes install bison build-essential cmake flex git libclang-common-"${LLVM_VERSION}"-dev libelf-dev libllvm"${LLVM_VERSION}" libz-dev lldb-"${LLVM_VERSION}" llvm-"${LLVM_VERSION}" llvm-"${LLVM_VERSION}"-dev llvm-"${LLVM_VERSION}"-runtime
fi
if [[ "${TRAVIS_DIST}" == "bionic" ]]; then
    sudo apt-get --yes install bison build-essential cmake flex libfl-dev git libclang-common-"${LLVM_VERSION}"-dev libelf-dev libllvm"${LLVM_VERSION}" libz-dev lldb-"${LLVM_VERSION}" llvm-"${LLVM_VERSION}" llvm-"${LLVM_VERSION}"-dev llvm-"${LLVM_VERSION}"-runtime
fi

## Optionally build/install BCC
if [ -n "${BCC_VERSION}" ]; then
    git clone https://github.com/iovisor/bcc || true
    cd bcc
    git checkout master
    git pull
    if [[ "${BCC_VERSION}" == "0.8.0" ]]; then
        git checkout remotes/origin/tag_v0.8.0
    fi
    if [[ "${BCC_VERSION}" == "0.9.0" ]]; then
        git checkout remotes/origin/tag_v0.9.0
    fi
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
    FEATURES="default"
fi

cargo build --features ${FEATURES}
cargo test --features ${FEATURES}
sudo timeout --signal 15 --preserve-status 5.0m target/debug/rezolus --config configs/example.toml
sudo timeout --signal 15 --preserve-status 5.0m target/debug/rezolus --config configs/ci.toml
cargo build --release --features ${FEATURES}
cargo test --release --features ${FEATURES}
sudo timeout --signal 15 --preserve-status 5.0m target/release/rezolus --config configs/example.toml
sudo timeout --signal 15 --preserve-status 5.0m target/release/rezolus --config configs/ci.toml
