// Copyright 2024 Jelly Terra
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0
// that can be found in the LICENSE file and https://mozilla.org/MPL/2.0/.

use err_rs::err;
use crate::{def_parser, tag_matches, unexpected_token};
use crate::ast::*;
use crate::parser::*;

macro_rules! range {
    ($begin: expr, $parser: expr) => {
        crate::scanner::PosRange { Begin: $begin, End: $parser.GetPos() }
    };
}

impl<T> List<T> where T: AstNodeParserTrait<T> {
    
    // List::Expect is special.
    // Parsing stops at the terminator, not one token after the terminator.
    //
    // This is designed to deal with grammars in which
    // the terminator of the second-level list is
    // the delimiter of the top-level list, such as:
    // ```
    // {
    //   a, b, c;
    //   x, y, z;
    //    ^     ^ top-level list delimiter, secondary list terminator
    //      secondary list delimiter
    // }
    // ^ top-level list terminator
    // ```
    pub fn Expect(p: &mut Parser, delimiter: TokenKind, terminator: TokenKind) -> Result<List<T>, ParserError> {
        let begin = p.GetPos();
        let mut list: Vec<T> = vec![];

        loop {
            if tag_matches!(&p.Token.Kind, &terminator) {
                // () <- terminator
                // (...,...,) <- terminator
                break;
            }
            list.push(T::Expect(p)?);
            if tag_matches!(&p.Token.Kind, &delimiter) {
                // (...,..., <- delimiter
                p.Scan()?; // delimiter
            } else {
                // (...,...) <- terminator
                p.Match(terminator.clone())?; // terminator
                break;
            }
        }

        Ok(List {
            Pos: range![begin, p],
            Elements: list,
            Delimiter: delimiter,
            Term: terminator,
        })
    }
}

def_parser! {
    Ident, p => {
        let token = p.GetTokenAndScan()?;
        match token.Kind {
            TokenKind::Ident => {
                Ident {
                    Pos: token.Pos,
                    Token: token.clone(),
                }
            }
            _ => {
                err!(ParserError::UnexpectedToken, UnexpectedTokenError {
                    Want: TokenKind::Ident,
                    Have: token.clone(),
                })
            }
        }
    },

    Field, p => {
        let begin = p.GetPos();
        p.MatchAndScan(TokenKind::FIELD)?;
        let name = Ident::Expect(p)?;
        p.MatchAndScan(TokenKind::COLON)?;
        let rule = Ident::Expect(p)?;
        
        Field {
            Name: name,
            Rule: rule,
            Pos: range![begin, p],
        }
    },
    
    Pattern, p => {
        let begin = p.GetPos();
        let ahead = Ident::Expect(p)?;
        p.MatchAndScan(TokenKind::ARROW)?;
        let rule = List::Expect(p, TokenKind::COMMA, TokenKind::SEMICOLON)?;
        // Delimiter of Branch::Expect
        
        Pattern {
            Ahead: ahead,
            Rule: rule,
            Pos: range![begin, p],
        }
    },
    
    Branch, p => {
        let begin = p.GetPos();
        p.MatchAndScan(TokenKind::LBRACE)?;
        let patterns = List::Expect(p, TokenKind::SEMICOLON, TokenKind::RBRACE)?;
        p.Scan()?;
        
        Branch {
            Patterns: patterns,
            Pos: range![begin, p],
        }
    },
    
    ListRule, p => {
        let begin = p.GetPos();
        p.MatchAndScan(TokenKind::LPAREN)?;
        let field = Field::Expect(p)?;
        p.MatchAndScan(TokenKind::COMMA)?;
        let delimiter = Ident::Expect(p)?;
        p.MatchAndScan(TokenKind::COMMA)?;
        let term = Ident::Expect(p)?;
        p.MatchAndScan(TokenKind::RPAREN)?;
        
        ListRule {
            Field: field,
            Delimiter: delimiter,
            Term: term,
            Pos: range![begin, p],
        }
    },
    
    Node, p => {
        match p.Token.Kind {
            TokenKind::Ident => Node::Ident(Box::new(Ident::Expect(p)?)),
            TokenKind::FIELD => Node::Field(Box::new(Field::Expect(p)?)),
            TokenKind::LBRACE => Node::Match(Box::new(Branch::Expect(p)?)),
            TokenKind::LPAREN => Node::ListRule(Box::new(ListRule::Expect(p)?)),
            _ => unexpected_token!(TokenKind::None, p.Token.clone())
        }
    },
    
    Def, p => {
        let begin = p.GetPos();
        let name = Ident::Expect(p)?;
        p.MatchAndScan(TokenKind::DEFINE)?;
        let rule = List::Expect(p, TokenKind::COMMA, TokenKind::SEMICOLON)?;
        
        Def {
            Name: name,
            Rule: rule,
            Pos: range![begin, p],
        }
    },
    
    File, p => {
        let begin = p.GetPos();
        let definitions = List::Expect(p, TokenKind::SEMICOLON, TokenKind::EOF)?;
        
        File {
            Definitions: definitions,
            Pos: range![begin, p],
        }
    }
}
