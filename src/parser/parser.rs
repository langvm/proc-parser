// Copyright 2024 Jelly Terra
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0
// that can be found in the LICENSE file and https://mozilla.org/MPL/2.0/.

use std::collections::HashMap;

use err_rs::*;

use crate::ast::{Token, TokenKind};
use crate::parser::*;
use crate::scanner::*;
use crate::tag_matches;
use crate::unexpected_token;

pub trait AstNodeParserTrait<T> {
    fn Expect(p: &mut Parser) -> Result<T, ParserError>;
}

#[derive(Debug)]
pub enum ParserError {
    ScannerError(BasicScannerError),
    UnexpectedToken(UnexpectedTokenError),
}

pub struct Parser {
    pub Scanner: BasicScanner,

    pub KeywordLookup: HashMap<String, TokenKind>,

    pub Token: Token,

    // Insert semicolon when true
    pub CompleteSemicolon: bool,
}

impl Parser {
    pub fn new(buffer: Vec<char>) -> Parser {
        Parser {
            Scanner: BasicScanner {
                BufferScanner: BufferScanner::new(buffer),
                Delimiters: vec!['(', ')', '[', ']', '{', '}', ',', ';', '/', '\n'],
                Whitespaces: vec![' ', '\t', '\r'],
            },
            KeywordLookup: TokenKind::KeywordLookup(),
            Token: Token::default(),

            CompleteSemicolon: false,
        }
    }

    pub fn GetPos(&self) -> Position { self.Scanner.GetPos() }

    pub fn Scan(&mut self) -> Result<&Token, ParserError> {
        let bt = on_err!(self.Scanner.Scan(), err => match err {
            BasicScannerError::EOF(_) => match self.Token.Kind {
                TokenKind::EOF => err!(ParserError::ScannerError(err)),
                _ => {
                    self.Token.Kind = TokenKind::EOF;
                    ok!(&self.Token);
                }
            }
            _ => err!(ParserError::ScannerError(err))
        });

        let literal = bt.Literal.iter().collect::<String>();

        // Determines whether BasicToken is a keyword, operator or delimiter.
        let kind = match bt.Kind {
            BasicTokenKind::Ident => {
                match self.KeywordLookup.get(&literal) {
                    Some(v) => v.to_owned(),
                    None => TokenKind::Ident
                }
            }
            BasicTokenKind::Operator => {
                match self.KeywordLookup.get(&literal) {
                    Some(v) => v.to_owned(),
                    None => TokenKind::Operator
                }
            }
            BasicTokenKind::Delimiter => {
                self.KeywordLookup.get(&literal).expect("implementation forgotten to add delimiter to the keyword lookup table").to_owned()
            }
            BasicTokenKind::Int(format) => TokenKind::Int(format),
            BasicTokenKind::Float => TokenKind::Float,
            BasicTokenKind::String => TokenKind::String,
            BasicTokenKind::Char => TokenKind::Char,
            BasicTokenKind::Comment => return self.Scan()
        };

        match kind {
            TokenKind::NEWLINE => {
                if self.CompleteSemicolon {
                    self.CompleteSemicolon = false;
                    self.Token = Token {
                        Pos: bt.Pos,
                        Kind: TokenKind::SEMICOLON,
                        Literal: ";".to_string(),
                    };
                    ok!(&self.Token);
                }
                return self.Scan();
            }
            TokenKind::Ident | TokenKind::Int(_) | TokenKind::RBRACE | TokenKind::RPAREN => {
                self.CompleteSemicolon = true;
            }
            _ => {
                self.CompleteSemicolon = false;
            }
        }

        self.Token = Token {
            Pos: bt.Pos,
            Kind: kind,
            Literal: bt.Literal.iter().collect(),
        };

        Ok(&self.Token)
    }

    pub fn GetTokenAndScan(&mut self) -> Result<Token, ParserError> {
        let tok = self.Token.clone();
        self.Scan()?;
        Ok((tok))
    }

    pub fn Match(&mut self, term: TokenKind) -> Result<(), ParserError> {
        let tok = &self.Token;
        if !tag_matches!(&tok.Kind, &term) {
            unexpected_token!(term, tok.clone());
        }
        Ok(())
    }

    pub fn MatchAndScan(&mut self, term: TokenKind) -> Result<&Token, ParserError> {
        self.Match(term)?;
        Ok(self.Scan()?)
    }
}
