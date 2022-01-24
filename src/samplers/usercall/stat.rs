// Copyright 2021 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::metrics::{Source, Statistic};

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct UsercallStatistic {
    pub stat_path: String,
}

impl Statistic for UsercallStatistic {
    fn name(&self) -> &str {
        &self.stat_path
    }

    fn source(&self) -> Source {
        Source::Counter
    }
}
