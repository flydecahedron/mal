use std::{
    collections::{HashMap, VecDeque},
    fmt::{self, Display},
};
extern crate thiserror;
use self::thiserror::Error;
use crate::types::Value;

#[derive(Error, Debug)]
enum Error {
    #[error("No next character: {0}")]
    NoNextCharacter(Token),
    #[error("Unknown character: {0}")]
    UnknownCharacter(Token),
    #[error("Unexpected character, reader bug!: {0}")]
    UnexpectedCharacter(Token),
    #[error("Invalid number: {0}")]
    InvalidNumber(Token),
    #[error("Unterminated string: {0}")]
    UnterminatedString(Token),
}

#[derive(Debug, PartialEq)]
enum TokenType {
    // Single character tokens
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Star,
    Equal,
    Tilde,
    AtSign,
    Backtick,
    SingleQuote,
    Slash,
    // Keywords
    Let,
    Fn,
    Quote,
    Quasiquote,
    Unquote,
    UnquoteSplicing,

    // Literals
    Number,
    String,

    // Other
    Identifier,
    EOF,
    Error,
}

impl Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
struct Token {
    token_type: TokenType,
    start: usize,
    end: usize,
    line: usize,
    value: Option<Value>,
}

impl Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
struct Lexer<'a> {
    input: &'a str,
    start: usize,
    current: usize,
    line: usize,
}

impl<'a> Lexer<'a> {
    // Constructs a new Reader
    fn new(input: &'a str) -> Lexer<'a> {
        Lexer {
            input,
            start: 0,
            current: 0,
            line: 1,
        }
    }

    // Advances the current position by one character
    fn advance(&mut self) {
        if let Some(c) = self.input.chars().nth(self.current) {
            if c == '\n' {
                self.line += 1;
            }
            self.current += c.len_utf8();
        }
    }

    // Returns the next character without moving the cursor
    fn peek(&self) -> Option<char> {
        self.input.chars().nth(self.current)
    }

    // Returns the character after the next character without moving the cursor
    fn peek_next(&self) -> Option<char> {
        self.input.chars().nth(self.current + 1)
    }

    // Returns the next character and moves the cursor
    fn next_char(&mut self) -> Option<char> {
        let c = self.input.chars().nth(self.current);
        self.advance();
        c
    }

    // Returns the next token, skipping whitespace, and comments which are started with ';'
    fn next(&mut self) -> Result<Token, Error> {
        if self.is_at_end() {
            return Ok(self.create_token(TokenType::EOF));
        }

        self.skip_whitespace();
        match self.next_char() {
            Some(c) => match c {
                '(' => Ok(self.create_token(TokenType::LeftParen)),
                ')' => Ok(self.create_token(TokenType::RightParen)),
                '{' => Ok(self.create_token(TokenType::LeftBrace)),
                '}' => Ok(self.create_token(TokenType::RightBrace)),
                '[' => Ok(self.create_token(TokenType::LeftBracket)),
                ']' => Ok(self.create_token(TokenType::RightBracket)),
                ',' => Ok(self.create_token(TokenType::Comma)),
                '.' => Ok(self.create_token(TokenType::Dot)),
                '-' => Ok(self.create_token(TokenType::Minus)),
                '+' => Ok(self.create_token(TokenType::Plus)),
                '\'' => Ok(self.create_token(TokenType::SingleQuote)),
                '*' => Ok(self.create_token(TokenType::Star)),
                '=' => Ok(self.create_token(TokenType::Equal)),
                '@' => Ok(self.create_token(TokenType::AtSign)),
                '"' => self.create_string(),
                ';' => Err(Error::UnexpectedCharacter(self.create_error())),
                _ => {
                    if c.is_digit(10) {
                        Ok(self.number())
                    } else if c.is_alphabetic() {
                        self.indentifier_or_keyword()
                    } else {
                        Err(Error::UnknownCharacter(self.create_error()))
                    }
                }
            },

            None => return Err(Error::NoNextCharacter(self.create_error())),
        }
    }

    fn create_string(&mut self) -> Result<Token, Error> {
        while let Some(c) = self.peek() {
            if self.is_at_end() {
                return Err(Error::UnterminatedString(self.create_error()));
            }
            // Check for escaped quotes
            if c == '\\' {
                self.advance();
                if let Some(c) = self.peek() {
                    if c == '"' {
                        self.advance();
                        continue;
                    }
                }
            } else if c == '"' {
                break;
            }
            self.advance();
        }

        // The closing quote
        self.advance();
        Ok(Token {
            token_type: TokenType::String,
            start: self.start,
            end: self.current,
            line: self.line,
            value: Some(Value::String(
                self.input[self.start + 1..self.current - 1].to_string(),
            )),
        })
    }

    // This function takes a starting character and a keyword to advance through the character stream to see if it an exact match. If the keyword is not a match, then the identifier is returned instead.
    fn indentifier_or_keyword(&mut self) -> Result<Token, Error> {
        while let Some(c) = self.peek() {
            if !c.is_alphanumeric() {
                break;
            }
            self.advance();
        }

        match &self.input[self.start..self.current] {
            "let" => Ok(self.create_symbol(TokenType::Let)),
            "fn" => Ok(self.create_symbol(TokenType::Fn)),
            "quote" => Ok(self.create_symbol(TokenType::Quote)),
            "quasiquote" => Ok(self.create_symbol(TokenType::Quasiquote)),
            "unquote" => Ok(self.create_symbol(TokenType::Unquote)),
            "unquote-splicing" => Ok(self.create_symbol(TokenType::UnquoteSplicing)),
            _ => Ok(self.create_symbol(TokenType::Identifier)),
        }
    }

    fn skip_comment(&mut self) {
        while let Some(c) = self.peek() {
            if c == '\n' {
                self.advance();
                self.line += 1;
                break;
            }
            self.advance();
        }
    }

    // Returns a token for a number
    fn number(&mut self) -> Token {
        while let Some(c) = self.peek() {
            if !c.is_ascii_digit() {
                break;
            }
            self.advance();
        }
        let mut token = self.create_token(TokenType::Number);
        token.value = Some(Value::Number(
            self.input[self.start..self.current].parse().unwrap(),
        ));
        token
    }

    // Creates a token at the current position
    fn create_token(&self, token_type: TokenType) -> Token {
        Token {
            token_type,
            start: self.start,
            end: self.current,
            line: self.line,
            value: None,
        }
    }

    fn create_error(&self) -> Token {
        Token {
            token_type: TokenType::Error,
            start: self.start,
            end: self.current,
            line: self.line,
            value: Some(Value::Symbol(
                self.input[self.start..self.current].to_string(),
            )),
        }
    }

    // Creates a symbol token at the current position
    fn create_symbol(&self, token_type: TokenType) -> Token {
        Token {
            token_type,
            start: self.start,
            end: self.current,
            line: self.line,
            value: Some(Value::Symbol(
                self.input[self.start..self.current].to_string(),
            )),
        }
    }

    // Skips all whitespace characters and sets the current position
    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c == '\n' {
                self.line += 1;
                self.advance();
                continue;
            } else if c.is_whitespace() {
                self.advance();
                continue;
            }
            // Also skip comments which start with a ';'
            else if c == ';' {
                self.skip_comment();
                continue;
            }
            self.start = self.current;
            break;
        }
    }

    // Returns whether the end of the input has been reached
    fn is_at_end(&self) -> bool {
        self.current >= self.input.len() || self.peek().is_none() || self.peek().unwrap() == '\0'
    }
}

fn tokenize(input: &str) -> Result<Vec<Token>, Error> {
    let mut lexer = Lexer::new(input);
    let mut tokens = Vec::new();
    while !lexer.is_at_end() {
        tokens.push(lexer.next()?);
    }
    Ok(tokens)
}

struct Parser {
    tokens: Vec<Token>,
    index: usize,
    ast: Value,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Parser {
        Parser {
            tokens,
            index: 0,
            ast: Value::Null,
        }
    }

    fn parse_form(&mut self) -> Result<Value, Error> {
        match self.peek() {
            Some(token) => match token.token_type {
                TokenType::LeftParen => {
                    self.advance();
                    self.parse_list()
                }
                _ => {
                    let res = self.parse_atom();
                    self.advance();
                    res
                }
            },
            None => return Ok(Value::Error("End of Tokens".to_string())),
        }
    }

    fn parse_list(&mut self) -> Result<Value, Error> {
        let mut list: VecDeque<Value> = VecDeque::new();
        while let Some(token) = self.peek() {
            if token.token_type == TokenType::RightParen {
                self.advance();
                break;
            }
            list.push_back(self.parse_form()?);
        }
        Ok(Value::List(list))
    }

    fn parse_atom(&mut self) -> Result<Value, Error> {
        // if let Some(token) = self.peek() {
        //     match token.token_type {
        //         TokenType::Number => Ok(Value::Number(self.tokens[self.index])),
        //         TokenType::String => Ok(Value::String(self.tokens[self.index])),
        //         TokenType::Symbol => Ok(Value::Symbol(self.tokens[self.index])),
        //         _ => Ok(Value::Null),
        //     }
        //     self.advance();
        // }
        // Err(Value::Error(format!(
        //     "Error parsing token: {:?}",
        //     self.peek()
        // )))

        match self.peek() {
            Some(token) => match token.token_type {
                TokenType::Number => Ok(token.value.as_ref().unwrap().clone()),
                TokenType::String => Ok(Value::String(
                    token.value.as_ref().unwrap().clone().to_string(),
                )),
                TokenType::Identifier => Ok(Value::Symbol(
                    token.value.as_ref().unwrap().clone().to_string(),
                )),
                _ => Ok(Value::Null),
            },
            None => Ok(Value::Null),
        }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.index)
    }

    fn advance(&mut self) {
        self.index += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_number() {
        let mut reader = Lexer::new("123");
        let token = reader.next().unwrap();
        assert_eq!(token.value.unwrap(), Value::Number(123.0));
    }

    #[test]
    fn test_string() {
        // TODO quote escaping in strings or at least figure how to properly test it here
        let mut reader = Lexer::new("\"hello\"");
        let token = reader.next().unwrap();
        assert_eq!(token.value.unwrap(), Value::String("hello".to_string()));
    }

    #[test]
    fn test_symbol() {
        let mut reader = Lexer::new("hello");
        let token = reader.next().unwrap();
        assert_eq!(token.value.unwrap(), Value::Symbol("hello".to_string()));
    }

    #[test]
    fn test_all_single_character_tokens() {
        let mut reader = Lexer::new("(){}[]*-+=.,");
        let token = reader.next().unwrap();
        assert_eq!(token.token_type, TokenType::LeftParen);
        let token = reader.next().unwrap();
        assert_eq!(token.token_type, TokenType::RightParen);
        let token = reader.next().unwrap();
        assert_eq!(token.token_type, TokenType::LeftBrace);
        let token = reader.next().unwrap();
        assert_eq!(token.token_type, TokenType::RightBrace);
        let token = reader.next().unwrap();
        assert_eq!(token.token_type, TokenType::LeftBracket);
        let token = reader.next().unwrap();
        assert_eq!(token.token_type, TokenType::RightBracket);
        let token = reader.next().unwrap();
        assert_eq!(token.token_type, TokenType::Star);
        let token = reader.next().unwrap();
        assert_eq!(token.token_type, TokenType::Minus);
        let token = reader.next().unwrap();
        assert_eq!(token.token_type, TokenType::Plus);
        let token = reader.next().unwrap();
        assert_eq!(token.token_type, TokenType::Equal);
        let token = reader.next().unwrap();
        assert_eq!(token.token_type, TokenType::Dot);
        let token = reader.next().unwrap();
        assert_eq!(token.token_type, TokenType::Comma);
        let token = reader.next().unwrap();
        assert_eq!(token.token_type, TokenType::EOF);
    }

    #[test]
    fn test_comments() {
        let mut reader = Lexer::new("; hello \t\n\r   sym");
        let token = reader.next().unwrap();
        assert_eq!(token.token_type, TokenType::Identifier);
        assert_eq!(token.value, Some(Value::Symbol("sym".to_string())));
    }

    #[test]
    fn test_tokenize() {
        let input = "(+ 1 2)";
        let tokens = tokenize(input).unwrap();
        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[0].token_type, TokenType::LeftParen);
        assert_eq!(tokens[1].token_type, TokenType::Plus);
        assert_eq!(tokens[2].token_type, TokenType::Number);
        assert_eq!(tokens[3].token_type, TokenType::Number);
        assert_eq!(tokens[4].token_type, TokenType::RightParen);

        let input = "\"hello\"";
        let tokens = tokenize(input).unwrap();
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].token_type, TokenType::String);
        assert_eq!(tokens[1].token_type, TokenType::EOF);

        let input = "(- 1 2) (* 1 2) (/ 1 2) (= 1 2) (> 1 2) (< 1 2) (>= 1 2) (<= 1 2)";
        let tokens = tokenize(input).unwrap();
        assert_eq!(tokens.len(), 17);
        assert_eq!(tokens[0].token_type, TokenType::LeftParen);
        assert_eq!(tokens[1].token_type, TokenType::Minus);
        assert_eq!(tokens[2].token_type, TokenType::Number);
        assert_eq!(tokens[3].token_type, TokenType::Number);
        assert_eq!(tokens[4].token_type, TokenType::RightParen);
        assert_eq!(tokens[5].token_type, TokenType::LeftParen);
        assert_eq!(tokens[6].token_type, TokenType::Star);
        assert_eq!(tokens[7].token_type, TokenType::Number);
        assert_eq!(tokens[8].token_type, TokenType::Number);
        assert_eq!(tokens[9].token_type, TokenType::RightParen);
        assert_eq!(tokens[10].token_type, TokenType::LeftParen);
        assert_eq!(tokens[11].token_type, TokenType::Slash);
        assert_eq!(tokens[12].token_type, TokenType::Number);
        assert_eq!(tokens[13].token_type, TokenType::Number);
        assert_eq!(tokens[14].token_type, TokenType::RightParen);
        assert_eq!(tokens[15].token_type, TokenType::LeftParen);
        assert_eq!(tokens[16].token_type, TokenType::Equal);
        assert_eq!(tokens[17].token_type, TokenType::Number);
        assert_eq!(tokens[18].token_type, TokenType::Number);
        assert_eq!(tokens[19].token_type, TokenType::RightParen);
        // assert_eq!(tokens[20].token_type, TokenType::LeftParen);
        // assert_eq!(tokens[21].token_type, TokenType::Greater);
        // assert_eq!(tokens[22].token_type, TokenType::Number);
        // assert_eq!(tokens[23].token_type, TokenType::Number);
        // assert_eq!(tokens[24].token_type, TokenType::RightParen);
        // assert_eq!(tokens[25].token_type, TokenType::LeftParen);
        // assert_eq!(tokens[26].token_type, TokenType::Less);
        // assert_eq!(tokens[27].token_type, TokenType::Number);
        // assert_eq!(tokens[28].token_type, TokenType::Number);
        // assert_eq!(tokens[29].token_type, TokenType::RightParen);
        // assert_eq!(tokens[30].token_type, TokenType::LeftParen);
        // assert_eq!(tokens[31].token_type, TokenType::GreaterEqual);
        // assert_eq!(tokens[32].token_type, TokenType::Number);
        // assert_eq!(tokens[33].token_type, TokenType::Number);
        // assert_eq!(tokens[34].token_type, TokenType::RightParen);
        // assert_eq!(tokens[35].token_type, TokenType::LeftParen);
        // assert_eq!(tokens[36].token_type, TokenType::LessEqual);
        // assert_eq!(tokens[37].token_type, TokenType::Number);
        // assert_eq!(tokens[38].token_type, TokenType::Number);
        // assert_eq!(tokens[39].token_type, TokenType::RightParen);
    }
}
