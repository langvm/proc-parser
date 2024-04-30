// Copyright 2024 Jelly Terra
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0
// that can be found in the LICENSE file and https://mozilla.org/MPL/2.0/.

use std::char::from_u32;
use std::fmt::{Debug, Formatter};

use err_rs::*;

use crate::scanner::*;

macro_rules! range {
    ($begin: expr, $scanner: expr) => {
        crate::scanner::PosRange { Begin: $begin, End: $scanner.GetPos() }
    };
}

macro_rules! collect_from_to {
    ($scanner: expr, $begin: expr) => {
		$scanner.BufferScanner.Buffer[$begin.Offset..$scanner.GetPos().Offset].to_owned()
	};
}

pub struct BufferScanner {
    pub Pos: Position,
    pub Buffer: Vec<char>,
}

pub struct EOFError {
    pub Pos: Position,
}

impl Debug for EOFError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: end of file", self.Pos)
    }
}

impl BufferScanner {
    pub fn GetChar(&self) -> Result<char, EOFError> {
        if self.Pos.Offset == self.Buffer.len() {
            return Err(EOFError {
                Pos: self.Pos,
            });
        }

        Ok(self.Buffer[self.Pos.Offset])
    }

    pub fn Move(&mut self) -> Result<char, EOFError> {
        let ch = self.GetChar()?;

        if ch == '\n' {
            self.Pos.Line += 1;
            self.Pos.Column = 0;
        } else {
            self.Pos.Column += 1;
        }
        self.Pos.Offset += 1;

        Ok(ch)
    }

    pub fn GotoNextLine(&mut self) -> Result<(), EOFError> {
        loop {
            match self.Move() {
                Ok(ch) => {
                    if ch == '\n' {
                        break;
                    }
                }
                Err(err) => { return Err(err); }
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum BasicScannerError {
    EOF(EOFError),
    BadFormat(BadFormatError),
}

pub struct BadFormatError {
    pub PosRange: PosRange,
}

impl Debug for BadFormatError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: format error", self.PosRange)
    }
}

pub struct BasicScanner {
    pub BufferScanner: BufferScanner,

    pub Delimiters: Vec<char>,
    pub Whitespaces: Vec<char>,
}

impl BufferScanner {
    pub fn new(buffer: Vec<char>) -> BufferScanner {
        BufferScanner {
            Pos: Position {
                Offset: 0,
                Line: 0,
                Column: 0,
            },
            Buffer: buffer,
        }
    }
}

impl BasicScanner {
    pub fn GetChar(&self) -> Result<char, BasicScannerError> { Ok(wrap_result!(BasicScannerError::EOF, self.BufferScanner.GetChar())) }

    pub fn Move(&mut self) -> Result<char, BasicScannerError> { Ok(wrap_result!(BasicScannerError::EOF, self.BufferScanner.Move())) }

    pub fn GotoNextLine(&mut self) -> Result<(), BasicScannerError> { Ok(wrap_result!(BasicScannerError::EOF, self.BufferScanner.GotoNextLine())) }

    pub fn GetPos(&self) -> Position { self.BufferScanner.Pos }

    pub fn SkipWhitespaces(&mut self) -> Result<(), BasicScannerError> {
        while self.Whitespaces.contains(&self.GetChar()?) {
            self.Move()?;
        }

        Ok(())
    }

    pub fn ScanLineComment(&mut self) -> Result<BasicToken, BasicScannerError> {
        let begin = self.GetPos();

        self.GotoNextLine()?;

        Ok(BasicToken {
            Pos: range![begin, self],
            Kind: BasicTokenKind::Comment,
            Literal: collect_from_to!(self, begin),
        })
    }

    pub fn ScanQuotedComment(&mut self) -> Result<BasicToken, BasicScannerError> {
        let begin = self.GetPos();

        loop {
            if self.Move()? == '*' {
                if self.Move()? == '/' {
                    break;
                }
            }
        }

        Ok(BasicToken {
            Pos: range![begin, self],
            Kind: BasicTokenKind::Comment,
            Literal: collect_from_to!(self, begin),
        })
    }

    pub fn ScanComment(&mut self) -> Result<BasicToken, BasicScannerError> {
        let begin = self.GetPos();

        return match self.Move()? {
            '/' => self.ScanLineComment(),
            '*' => self.ScanQuotedComment(),
            _ => {
                err!(BasicScannerError::BadFormat, BadFormatError {
                    PosRange: range![begin, self],
                })
            }
        };
    }

    pub fn ScanIdent(&mut self) -> Result<BasicToken, BasicScannerError> {
        let begin = self.GetPos();

        loop {
            let ch = self.GetChar()?;
            if ch.is_ascii_alphabetic() || ch.is_numeric() || ch == '_' {
                self.Move()?;
            } else {
                break;
            }
        }

        Ok(BasicToken {
            Pos: range![begin, self],
            Kind: BasicTokenKind::Ident,
            Literal: collect_from_to!(self, begin),
        })
    }

    pub fn ScanHex(&mut self) -> Result<BasicToken, BasicScannerError> {
        let begin = self.GetPos();

        loop {
            let ch = self.GetChar()?;
            if '0' <= ch && ch <= '9' || 'a' <= ch && ch <= 'f' {
                self.Move()?;
            } else {
                break;
            }
        }

        Ok(BasicToken {
            Pos: range![begin, self],
            Kind: BasicTokenKind::Int(IntFormat::HEX),
            Literal: collect_from_to!(self, begin),
        })
    }

    pub fn ScanDec(&mut self) -> Result<BasicToken, BasicScannerError> {
        let begin = self.GetPos();

        loop {
            let ch = self.GetChar()?;
            if '0' <= ch && ch <= '9' {
                self.Move()?;
            } else {
                break;
            }
        }

        Ok(BasicToken {
            Pos: range![begin, self],
            Kind: BasicTokenKind::Int(IntFormat::DEC),
            Literal: collect_from_to!(self, begin),
        })
    }

    pub fn ScanOct(&mut self) -> Result<BasicToken, BasicScannerError> {
        let begin = self.GetPos();

        loop {
            let ch = self.GetChar()?;
            if '0' <= ch && ch <= '7' {
                self.Move()?;
            } else {
                break;
            }
        }

        Ok(BasicToken {
            Pos: range![begin, self],
            Kind: BasicTokenKind::Int(IntFormat::OCT),
            Literal: collect_from_to!(self, begin),
        })
    }

    pub fn ScanBin(&mut self) -> Result<BasicToken, BasicScannerError> {
        let begin = self.GetPos();

        loop {
            let ch = self.GetChar()?;
            if ch == '0' || ch == '1' {
                self.Move()?;
            } else {
                break;
            }
        }

        Ok(BasicToken {
            Pos: range![begin, self],
            Kind: BasicTokenKind::Int(IntFormat::BIN),
            Literal: collect_from_to!(self, begin),
        })
    }

    pub fn ScanDigit(&mut self) -> Result<BasicToken, BasicScannerError> {
        let begin = self.GetPos();

        match self.Move()? {
            '0' => {
                match self.Move()? {
                    'x' => self.ScanHex(),
                    'o' => self.ScanOct(),
                    'b' => self.ScanBin(),
                    _ => {
                        err!(BasicScannerError::BadFormat, BadFormatError {
                            PosRange: PosRange {
                                Begin: begin,
                                End: self.GetPos(),
                            },
                        })
                    }
                }
            }
            _ => {
                self.BufferScanner.Pos = begin;
                self.ScanDec()
            }
        }
    }

    pub fn ScanUnicodeHex(&mut self, runesN: u8) -> Result<char, BasicScannerError> {
        let begin = self.GetPos();

        let mut seq: Vec<char> = vec![];
        for _ in 0..runesN {
            seq.push(self.Move()?);
        }

        let ch = match u32::from_str_radix(&String::from_iter(seq), 16) {
            Ok(ch) => {
                match from_u32(ch) {
                    None => {
                        err!(BasicScannerError::BadFormat, BadFormatError {
                            PosRange: range![begin, self]
                        })
                    }
                    Some(ch) => ch
                }
            }
            Err(err) => {
                err!(BasicScannerError::BadFormat, BadFormatError {
                    PosRange: range![begin, self]
                })
            }
        };

        Ok(ch)
    }

    pub fn ScanEscapeChar(&mut self, quote: char) -> Result<char, BasicScannerError> {
        let begin = self.GetPos();

        let ch = self.Move()?;

        Ok(match ch {
            'n' => '\n',
            't' => '\t',
            'r' => '\r',
            '\\' => '\\',
            'x' => self.ScanUnicodeHex(2)?, // 1 byte
            'u' => self.ScanUnicodeHex(4)?, // 2 byte
            'U' => self.ScanUnicodeHex(8)?, // 4 byte
            _ if ch == quote => quote,
            _ => err!(BasicScannerError::BadFormat, BadFormatError { PosRange: range![begin, self] })
        })
    }

    pub fn ScanString(&mut self, quote: char) -> Result<BasicToken, BasicScannerError> {
        let begin = self.GetPos();

        self.Move()?; // skip quote

        let mut seq: Vec<char> = vec![];

        loop {
            let ch = self.Move()?;
            match ch {
                '\\' => {
                    let esc = self.ScanEscapeChar(quote)?;
                    seq.push(esc)
                }
                _ if ch == quote => break,
                _ => seq.push(ch)
            }
        }

        Ok(BasicToken {
            Pos: range![begin, self],
            Kind: BasicTokenKind::String,
            Literal: seq,
        })
    }

    pub fn ScanOperator(&mut self) -> Result<BasicToken, BasicScannerError> {
        let begin = self.GetPos();

        loop {
            match self.GetChar()? {
                '"' => break,
                '\'' => break,
                ch if !ch.is_ascii_punctuation() => break,
                ch if self.Delimiters.contains(&ch) => break,
                _ => self.Move()?
            };
        }

        Ok(BasicToken {
            Pos: range![begin, self],
            Kind: BasicTokenKind::Operator,
            Literal: collect_from_to!(self, begin),
        })
    }

    pub fn Scan(&mut self) -> Result<BasicToken, BasicScannerError> {
        self.SkipWhitespaces()?;

        let begin = self.GetPos();

        match self.GetChar()? {
            ch if ch.is_alphabetic() => self.ScanIdent(),
            ch if ch.is_numeric() => self.ScanDigit(),
            ch if self.Delimiters.contains(&ch) => Ok(
                BasicToken {
                    Pos: range![begin, self],
                    Kind: BasicTokenKind::Delimiter,
                    Literal: vec![self.Move()?],
                }
            ),
            '_' => self.ScanIdent(),
            '"' => self.ScanString('"'),
            '\'' => self.ScanString('\''),
            '/' => self.ScanComment(),
            ch if ch.is_ascii_punctuation() => self.ScanOperator(),
            _ => err!(BasicScannerError::EOF, EOFError { Pos: self.GetPos() })
        }
    }
}
