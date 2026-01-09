use crate::token::{Token, TokenType, lookup_identifier};

pub struct Lexer {
    input: Vec<char>,
    position: usize,
    read_position: usize,
    ch: char,
}

impl Lexer {
    pub fn new(input: impl AsRef<str>) -> Self {
        let mut lexer = Lexer {
            input: input.as_ref().chars().collect(),
            position: 0,
            read_position: 0,
            ch: '\0',
        };

        lexer.read_char();
        lexer
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        let token = match self.ch {
            '(' => Token::new(TokenType::LParen, self.ch.to_string()),
            ')' => Token::new(TokenType::RParen, self.ch.to_string()),
            '{' => Token::new(TokenType::LBrace, self.ch.to_string()),
            '}' => Token::new(TokenType::RBrace, self.ch.to_string()),
            '[' => Token::new(TokenType::LBrack, self.ch.to_string()),
            ']' => Token::new(TokenType::RBrack, self.ch.to_string()),
            ',' => Token::new(TokenType::Comma, self.ch.to_string()),
            ';' => Token::new(TokenType::Semicolon, self.ch.to_string()),
            '.' => Token::new(TokenType::Dot, self.ch.to_string()),
            '+' => Token::new(TokenType::Plus, self.ch.to_string()),
            '-' => Token::new(TokenType::Minus, self.ch.to_string()),
            '/' => Token::new(TokenType::Slash, self.ch.to_string()),
            '#' => Token::new(TokenType::Comment, self.read_comment()),
            '\0' => Token::new(TokenType::EOF, String::new()),
            '"' => {
                let literal = self.read_string();

                if self.ch == '"' {
                    Token::new(TokenType::String, literal)
                } else {
                    Token::new(
                        TokenType::Illegal,
                        format!("unterminated string: \"{}\"", literal),
                    )
                }
            }
            '=' => match self.peek_char() {
                '=' => Token::new(TokenType::Eq, self.read_operator()),
                '>' => Token::new(TokenType::HashRocket, self.read_operator()),
                _ => Token::new(TokenType::Assign, self.ch.to_string()),
            },
            '!' => match self.peek_char() {
                '=' => Token::new(TokenType::NotEq, self.read_operator()),
                _ => Token::new(TokenType::Bang, self.ch.to_string()),
            },
            '<' => match self.peek_char() {
                '=' => Token::new(TokenType::LTE, self.read_operator()),
                _ => Token::new(TokenType::LT, self.ch.to_string()),
            },
            '>' => match self.peek_char() {
                '=' => Token::new(TokenType::GTE, self.read_operator()),
                _ => Token::new(TokenType::GT, self.ch.to_string()),
            },
            '*' => match self.peek_char() {
                '*' => Token::new(TokenType::Power, self.read_operator()),
                _ => Token::new(TokenType::Asterisk, self.ch.to_string()),
            },
            '&' => match self.peek_char() {
                '&' => Token::new(TokenType::And, self.read_operator()),
                _ => Token::new(TokenType::Illegal, self.ch.to_string()),
            },
            '|' => match self.peek_char() {
                '|' => Token::new(TokenType::Or, self.read_operator()),
                _ => Token::new(TokenType::Illegal, self.ch.to_string()),
            },
            ':' => {
                self.read_char();

                if Self::is_letter(self.ch) {
                    return Token::new(TokenType::Symbol, format!(":{}", self.read_identifier()));
                } else {
                    Token::new(TokenType::Illegal, self.ch.to_string())
                }
            }
            _ => {
                if Self::is_letter(self.ch) {
                    let literal = self.read_identifier();

                    if self.ch == ':' {
                        self.read_char(); // Advance past the colon
                        return Token::new(TokenType::SymbolKey, format!("{}:", literal));
                    }

                    return Token::new(lookup_identifier(&literal), literal);
                } else if Self::is_digit(self.ch) {
                    let (token_type, literal) = self.read_number();
                    return Token::new(token_type, literal);
                } else {
                    Token::new(TokenType::Illegal, self.ch.to_string())
                }
            }
        };

        self.read_char();
        token
    }

    fn skip_whitespace(&mut self) {
        let ignoreable_whitespaces = [' ', '\t', '\n', '\r'];

        while ignoreable_whitespaces.contains(&self.ch) {
            self.read_char();
        }
    }

    fn read_char(&mut self) {
        if self.input.len() <= self.read_position {
            self.ch = '\0';
        } else {
            self.ch = self.input[self.read_position];
        }

        self.position = self.read_position;
        self.read_position += 1;
    }

    fn peek_char(&self) -> char {
        if self.input.len() <= self.read_position {
            '\0'
        } else {
            self.input[self.read_position]
        }
    }

    fn read_operator(&mut self) -> String {
        let first_char = self.ch;
        self.read_char();

        format!("{}{}", first_char, self.ch)
    }

    fn read_comment(&mut self) -> String {
        let mut position = self.position + 1; // Skip the '#' character

        self.read_char();

        while self.ch == ' ' {
            position += 1;
            self.read_char();
        }

        while self.ch != '\n' && self.ch != '\0' {
            self.read_char();
        }

        self.input[position..self.position].iter().collect()
    }

    fn read_string(&mut self) -> String {
        let position = self.position + 1; // Skip the opening quote

        self.read_char();

        while self.ch != '"' && self.ch != '\0' {
            self.read_char();
        }

        let str_literal = self.input[position..self.position].iter().collect();

        str_literal
    }

    fn read_identifier(&mut self) -> String {
        let position = self.position;

        while Self::is_letter(self.ch) {
            self.read_char();
        }

        self.input[position..self.position].iter().collect()
    }

    fn is_letter(ch: char) -> bool {
        ch.is_alphabetic() || ch == '_'
    }

    fn is_digit(ch: char) -> bool {
        ch.is_digit(10)
    }

    fn read_number(&mut self) -> (TokenType, String) {
        let position = self.position;

        while Self::is_digit(self.ch) {
            self.read_char();
        }

        if self.ch == '.' && Self::is_digit(self.peek_char()) {
            self.read_char();

            while Self::is_digit(self.ch) {
                self.read_char();
            }

            (
                TokenType::Float,
                self.input[position..self.position].iter().collect(),
            )
        } else {
            (
                TokenType::Integer,
                self.input[position..self.position].iter().collect(),
            )
        }
    }
}
