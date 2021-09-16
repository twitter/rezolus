// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use anyhow::Result;

fn main() -> Result<()> {
    vergen::vergen(vergen::Config::default())
}
