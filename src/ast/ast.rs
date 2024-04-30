// Copyright 2024 Jelly Terra
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0
// that can be found in the LICENSE file and https://mozilla.org/MPL/2.0/.

use std::fmt;

use crate::ast::*;
use crate::scanner::*;

macro_rules! def_ast {
    (
        $(
        $ast:ident {
            $($name:ident: $typ:ty), *,
        }
        ), *
    ) => {
        $(
        #[derive(Default)]
        pub struct $ast {
            pub Pos: PosRange,
            $(
            pub $name: $typ,
            )*
        }
        )*
    };
}

macro_rules! def_node {
    (
        $(
        $node:ident {
            $($typ:ident), *,
        }
        ), *
    ) => {
        $(
        pub enum $node {
            None,
            $(
            $typ(Box<$typ>),
            )*
        }
        impl Default for $node {
            fn default() -> Self {
                $node::None
            }
        }
        impl std::fmt::Display for $node {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $node::None => { write!(f, "") }
                    $(
                    $node::$typ(e) => { write!(f, "{}", e) }
                    )*
                }
            }
        }
        )*
    };
}

#[derive(Default)]
pub struct List<T> {
    pub Pos: PosRange,
    pub Elements: Vec<T>,
    pub Delimiter: TokenKind,
    pub Term: TokenKind,
}

impl<T> fmt::Display for List<T> where T: fmt::Display {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for e in &self.Elements {
            write!(f, "{}{}", e, self.Delimiter)?;
        }
        Ok(())
    }
}

pub enum Node {
    None,
    Ident(Box<Ident>),
    Field(Box<Field>),
    Match(Box<Branch>),
    ListRule(Box<ListRule>),
}

impl Default for Node { fn default() -> Self { Node::None } }

pub enum Optional<T> {
    None,
    Some(T),
}

impl<T> Default for Optional<T> {
    fn default() -> Self { Optional::None }
}

impl<T> fmt::Display for Optional<T> where T: fmt::Display {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Optional::Some(v) => { write!(f, "{}", v) }
            Optional::None => { Ok(()) }
        }
    }
}

def_ast! {
    Ident {
        Token: Token,
    },

    Field {
        Name: Ident,
        Rule: Ident,
    },
    
    Pattern {
        Ahead: Ident,
        Rule: List<Node>,
    },
    
    Branch {
        Patterns: List<Pattern>,
    },

    ListRule {
        Field: Field,
        Delimiter: Ident,
        Term: Ident,
    },
    
    Def {
        Name: Ident,
        Rule: List<Node>,
    },
    
    File {
        Definitions: List<Def>,
    }
}
