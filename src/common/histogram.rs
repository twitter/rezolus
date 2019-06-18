// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

pub fn key_to_value1(index: u64) -> Option<u64> {
    if index <= 10 {
        Some(index)
    } else if index < 20 {
        Some((index - 9) * 10)
    } else if index < 30 {
        Some((index - 19) * 100)
    } else if index < 40 {
        Some((index - 29) * 1000)
    } else if index < 50 {
        Some((index - 39) * 10000)
    } else if index < 60 {
        Some((index - 49) * 100000)
    } else {
        None
    }
}

// reverse indexing for use in userspace
pub fn key_to_value2(index: u64) -> Option<u64> {
    if index < 100 {
        Some(index)
    } else if index < 200 {
        Some(100 + (index - 99) * 10 - 1)
    } else if index < 300 {
        Some((index - 200) * 100 + 1099)
    } else if index < 400 {
        Some((index - 300) * 10_000_000)
    } else if index < 500 {
        Some((index - 399) * 1_000_000_000)
    } else if index < 600 {
        Some((index - 499) * 100_000_000_000)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_to_value1() {
        assert_eq!(Some(0), key_to_value1(0));
        assert_eq!(Some(8), key_to_value1(8));
        assert_eq!(Some(9), key_to_value1(9));
        assert_eq!(Some(10), key_to_value1(10));
        assert_eq!(Some(20), key_to_value1(11));
        assert_eq!(Some(90), key_to_value1(18));
        assert_eq!(Some(100), key_to_value1(19));
        assert_eq!(Some(900), key_to_value1(28));
        assert_eq!(Some(1000), key_to_value1(29));
        assert_eq!(Some(9000), key_to_value1(38));
        assert_eq!(Some(10_000), key_to_value1(39));
        assert_eq!(Some(100_000), key_to_value1(49));
        assert_eq!(Some(1_000_000), key_to_value1(59));
    }

    #[test]
    fn test_key_to_value2() {
        assert_eq!(Some(0), key_to_value2(0));
        assert_eq!(Some(1), key_to_value2(1));
        assert_eq!(Some(99), key_to_value2(99));
        assert_eq!(Some(109), key_to_value2(100));
        assert_eq!(Some(119), key_to_value2(101));
        assert_eq!(Some(129), key_to_value2(102));
        assert_eq!(Some(999), key_to_value2(199));
        assert_eq!(Some(1099), key_to_value2(200));
        assert_eq!(Some(1199), key_to_value2(201));
        assert_eq!(Some(999_000), key_to_value2(299));
        assert_eq!(Some(1_000_000), key_to_value2(300));
        assert_eq!(Some(1_100_000), key_to_value2(301));
        assert_eq!(Some(99_000_000), key_to_value2(399));
        assert_eq!(Some(100_000_000), key_to_value2(400));
        assert_eq!(Some(110_000_000), key_to_value2(401));
        assert_eq!(Some(990_000_000), key_to_value2(499));
        assert_eq!(Some(1_000_000_000), key_to_value2(500));
        assert_eq!(Some(1_100_000_000), key_to_value2(501));
        assert_eq!(Some(99_000_000_000), key_to_value2(599));
    }
}
