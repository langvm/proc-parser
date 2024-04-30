// Copyright 2024 Jelly Terra
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0
// that can be found in the LICENSE file and https://mozilla.org/MPL/2.0/.

use std::fmt;
use std::fmt::Formatter;

use crate::ast::{Node, Token, TokenKind};

pub struct UnexpectedTokenError {
    pub Want: TokenKind,
    pub Have: Token,
}

impl fmt::Debug for UnexpectedTokenError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result { write!(f, "{} unexpected token: want {} but have {} \"{}\"", self.Have.Pos, self.Want, self.Have.Kind, self.Have.Literal) }
}
