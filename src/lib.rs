// Copyright 2024 Jelly Terra
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0
// that can be found in the LICENSE file and https://mozilla.org/MPL/2.0/.

#![allow(nonstandard_style)]
#![allow(warnings)]

pub mod parser;
pub mod scanner;
pub mod ast;

#[macro_export]
macro_rules! tag_matches {
    ($e:expr, $p:expr) => {
        std::mem::discriminant($e) == std::mem::discriminant($p)
    };
}
