// Copyright 2024 Jelly Terra
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0
// that can be found in the LICENSE file and https://mozilla.org/MPL/2.0/.

#[macro_export]
macro_rules! def_parser {
    (
        $(
        $ast_node:ty, $p:ident => $block:block
        ), *
    ) => {
        $(
        impl crate::parser::AstNodeParserTrait<$ast_node> for $ast_node {
            fn Expect($p: &mut crate::parser::Parser) -> Result<$ast_node, ParserError> { Ok($block) }
        }
        )*
    };
}

#[macro_export]
macro_rules! unexpected_token {
    ($want:expr, $have:expr) => {
        err_rs::err!(crate::parser::ParserError::UnexpectedToken, crate::parser::UnexpectedTokenError {
            Want: $want,
            Have: $have,
        })
    }
}
