#!/bin/bash -ev

set -e

## Update apt
sudo apt-get update

## Install kernel headers for matching version
sudo apt-get install linux-headers-"$(uname -r)"
sudo apt-get remove *llvm* *clang*
sudo apt-get --yes install clang-"${LLVM}" libclang-"${LLVM}"-dev libelf-dev \
    libfl-dev llvm-"${LLVM}"-dev libz-dev llvm-"${LLVM}"

# For static builds, we need to compile the following
if [[ $STATIC == true ]]; then
    export CPPFLAGS="-P"
    export CFLAGS="-fPIC"

    echo "build binutils"
    curl -L -O ftp://sourceware.org/pub/binutils/snapshots/binutils-2.34.90.tar.xz
    tar xf binutils-2.34.90.tar.xz
    cd binutils-2.34.90
    ./configure --prefix=/usr
    make -j2
    sudo make install
    cd ..

    echo "build zlib"
    curl -L -O https://zlib.net/zlib-1.2.11.tar.gz
    tar xzf zlib-1.2.11.tar.gz
    cd zlib-1.2.11
    ./configure --prefix=/usr
    make -j2
    sudo make install
    cd ..

    echo "build xz"
    curl -L -O https://tukaani.org/xz/xz-5.2.5.tar.gz
    tar xzf xz-5.2.5.tar.gz
    cd xz-5.2.5
    ./configure --prefix=/usr
    make -j2
    sudo make install
    cd ..

    echo "build ncurses"
    curl -L -O ftp://ftp.invisible-island.net/ncurses/ncurses-6.2.tar.gz
    tar xzf ncurses-6.2.tar.gz
    cd ncurses-6.2
    ./configure --prefix=/usr --with-termlib
    make -j2
    sudo make install
    cd ..

    echo "build libxml2"
    git clone https://gitlab.gnome.org/GNOME/libxml2
    cd libxml2
    git checkout 41a34e1f4ffae2ce401600dbb5fe43f8fe402641
    autoreconf -fvi
    ./configure --prefix=/usr --without-python
    make -j2
    sudo make install
    cd ..

    echo "build elfutils"
    curl -L -O ftp://sourceware.org/pub/elfutils/0.180/elfutils-0.180.tar.bz2
    tar xjf elfutils-0.180.tar.bz2
    cd elfutils-0.180
    ./configure --prefix=/usr --disable-debuginfod
    make -j2
    sudo make install
    cd ..
fi

## build/install BCC
git clone https://github.com/iovisor/bcc || true
cd bcc
git checkout master
git pull
if [[ "${BCC}" == "0.4.0" ]]; then
    git checkout remotes/origin/tag_v0.4.0
fi
if [[ "${BCC}" == "0.5.0" ]]; then
    git checkout remotes/origin/tag_v0.5.0
fi
if [[ "${BCC}" == "0.6.0" ]]; then
    git checkout remotes/origin/tag_v0.6.0
fi
if [[ "${BCC}" == "0.6.1" ]]; then
    git checkout remotes/origin/tag_v0.6.1
fi
if [[ "${BCC}" == "0.7.0" ]]; then
    git checkout remotes/origin/tag_v0.7.0
fi
if [[ "${BCC}" == "0.8.0" ]]; then
    git checkout remotes/origin/tag_v0.8.0
fi
if [[ "${BCC}" == "0.9.0" ]]; then
    git checkout remotes/origin/tag_v0.9.0
fi
if [[ "${BCC}" == "0.10.0" ]]; then
    git checkout remotes/origin/tag_v0.10.0
fi
if [[ "${BCC}" == "0.11.0" ]]; then
    git checkout 0fa419a64e71984d42f107c210d3d3f0cc82d59a
fi
if [[ "${BCC}" == "0.12.0" ]]; then
    git checkout 368a5b0714961953f3e3f61607fa16cb71449c1b
fi
if [[ "${BCC}" == "0.13.0" ]]; then
    git checkout 942227484d3207f6a42103674001ef01fb5335a0
fi
if [[ "${BCC}" == "0.14.0" ]]; then
    git checkout ceb458d6a07a42d8d6d3c16a3b8e387b5131d610
fi
if [[ "${BCC}" == "0.15.0" ]]; then
    git checkout e41f7a3be5c8114ef6a0990e50c2fbabea0e928e
fi
if [[ "${BCC}" == "0.16.0" ]]; then
    git checkout fecd934a9c0ff581890d218ff6c5101694e9b326
fi
mkdir -p _build
cd _build
cmake .. -DCMAKE_INSTALL_PREFIX=/usr
make -j2
sudo make install
find . -name "*.a" -exec sudo cp -v {} /usr/lib/ \;
cd ../..

## Build and test
function build {
    if [ -n "${FEATURES}" ]; then
        cargo build --release --features "${FEATURES}"
        cargo test --release --features "${FEATURES}"
    else
        cargo build --release
        cargo test --release
    fi
}

function static_build {
    export RUSTFLAGS="-L /usr/lib -L /usr/lib64 -L /usr/lib/llvm-${LLVM}/lib"

    if [ "${LLVM}" == "8" ]; then
        export FLAG="bpf_static_llvm_8";
    else 
        export FLAG="bpf_static";
    fi
    
    if [ -n "${FEATURES}" ]; then
        cargo build --release --features "${FEATURES} ${FLAG}"
        cargo test --release --features "${FEATURES} ${FLAG}"
    else
        cargo build --release --features "${FLAG}"
        cargo test --release --features "${FLAG}"
    fi
}

function test {
    sudo timeout --signal 15 --preserve-status 5.0m target/release/rezolus --config configs/example.toml &
    sleep 180
    curl -s http://localhost:4242/vars
    curl -s http://localhost:4242/vars.json | jq '.' > /dev/null
    sleep 180

    sudo timeout --signal 15 --preserve-status 5.0m target/release/rezolus --config configs/ci.toml &
    sleep 180
    curl -s http://localhost:4242/vars
    curl -s http://localhost:4242/vars.json | jq '.' > /dev/null
    sleep 180
}

build
test

if [[ $STATIC == true ]]; then
    static_build
    test
fi