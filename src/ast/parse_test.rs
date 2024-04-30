// Copyright 2024 Jelly Terra
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0
// that can be found in the LICENSE file and https://mozilla.org/MPL/2.0/.

use std::fs;

use embed_rs::embed_as_string;

use crate::ast::{Def, List, TokenKind};
use crate::parser::{AstNodeParserTrait, Parser};

#[test]
fn TestParser_Lex() {
    let src = fs::read_to_string("proc-parser.ppg").unwrap();

    let mut p = Parser::new(String::from(src).chars().collect());

    loop {
        let token = p.Scan().unwrap();
        match token.Kind {
            TokenKind::EOF => break,
            _ => println!("{} {}", token.Pos, token.Literal)
        };
    }
}

#[test]
fn TestParser_Expect() {
    let src = fs::read_to_string("proc-parser.ppg").unwrap();

    let mut p = Parser::new(String::from(src).chars().collect());

    p.Scan().unwrap();

    List::<Def>::Expect(&mut p, TokenKind::SEMICOLON, TokenKind::EOF).unwrap();
}
