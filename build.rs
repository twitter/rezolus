// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

fn main() {
    let mut flags = vergen::ConstantsFlags::all();
    flags.toggle(vergen::ConstantsFlags::SEMVER_FROM_CARGO_PKG);
    vergen::generate_cargo_keys(vergen::ConstantsFlags::all())
        .expect("Unable to generate the cargo keys!");
}
