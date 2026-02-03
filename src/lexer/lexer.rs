use crate::lexer::error::{LexerError, LexerErrorType};
use crate::token::{Token, TokenType, lookup_identifier};

const ESCAPABLE_CHARS: [char; 8] = ['n', 'r', 't', '\\', '"', '0', 'a', 'b'];

pub struct Lexer<'a> {
    // Hot fields - accessed every character
    ch: char,
    position: usize,
    read_position: usize,

    // Warm fields - accessed frequently
    line: usize,
    column: usize,

    // Cold fields - accessed less frequently
    input: &'a str,
    errors: Vec<LexerError>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        let mut lexer = Lexer {
            input,
            position: 0,
            read_position: 0,
            ch: '\0',
            line: 1,
            column: 0,
            errors: Vec::new(),
        };

        lexer.read_char();
        lexer
    }

    /// Returns the current line number.
    pub fn line(&self) -> usize {
        self.line
    }

    /// Returns the current column number.
    pub fn column(&self) -> usize {
        self.column
    }

    /// Returns a slice of all errors encountered during lexing.
    pub fn errors(&self) -> &[LexerError] {
        &self.errors
    }

    /// Returns the next token from the input.
    /// Note: Simple tokens rely on the trailing `self.read_char()` call,
    /// while complex tokens (strings, identifiers, numbers) intentionally return early.
    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        let token = match self.ch {
            '\0' => Token::new(TokenType::EOF, self.position, self.position),
            '(' => Token::new(TokenType::LParen, self.position, self.position + 1),
            ')' => Token::new(TokenType::RParen, self.position, self.position + 1),
            '{' => Token::new(TokenType::LBrace, self.position, self.position + 1),
            '}' => Token::new(TokenType::RBrace, self.position, self.position + 1),
            '[' => Token::new(TokenType::LBrack, self.position, self.position + 1),
            ']' => Token::new(TokenType::RBrack, self.position, self.position + 1),
            ',' => Token::new(TokenType::Comma, self.position, self.position + 1),
            ';' => Token::new(TokenType::Semicolon, self.position, self.position + 1),
            '.' => Token::new(TokenType::Dot, self.position, self.position + 1),
            '+' => Token::new(TokenType::Plus, self.position, self.position + 1),
            '-' => Token::new(TokenType::Minus, self.position, self.position + 1),
            '/' => Token::new(TokenType::Slash, self.position, self.position + 1),
            '#' => Token::new(TokenType::Comment, self.position, self.read_comment()),
            '=' => match self.peek_ascii_char() {
                '=' => Token::new(TokenType::Eq, self.position, self.read_operator()),
                '>' => Token::new(TokenType::HashRocket, self.position, self.read_operator()),
                _ => Token::new(TokenType::Assign, self.position, self.position + 1),
            },
            '!' => match self.peek_ascii_char() {
                '=' => Token::new(TokenType::NotEq, self.position, self.read_operator()),
                _ => Token::new(TokenType::Bang, self.position, self.position + 1),
            },
            '<' => match self.peek_ascii_char() {
                '=' => Token::new(TokenType::LTE, self.position, self.read_operator()),
                _ => Token::new(TokenType::LT, self.position, self.position + 1),
            },
            '>' => match self.peek_ascii_char() {
                '=' => Token::new(TokenType::GTE, self.position, self.read_operator()),
                _ => Token::new(TokenType::GT, self.position, self.position + 1),
            },
            '*' => match self.peek_ascii_char() {
                '*' => Token::new(TokenType::Power, self.position, self.read_operator()),
                _ => Token::new(TokenType::Asterisk, self.position, self.position + 1),
            },
            '&' => match self.peek_ascii_char() {
                '&' => Token::new(TokenType::And, self.position, self.read_operator()),
                _ => self.record_illegal_token(
                    self.position,
                    self.position + 1,
                    LexerErrorType::UnexpectedCharacter,
                ),
            },
            '|' => match self.peek_ascii_char() {
                '|' => Token::new(TokenType::Or, self.position, self.read_operator()),
                _ => self.record_illegal_token(
                    self.position,
                    self.position + 1,
                    LexerErrorType::UnexpectedCharacter,
                ),
            },
            '"' => {
                return match self.read_string() {
                    Ok(start) => Token::new(TokenType::String, start, self.position),
                    Err((error_type, start)) => {
                        self.record_illegal_token(start, self.position, error_type)
                    }
                };
            }
            ':' => {
                return match self.read_symbol() {
                    Ok(start) => Token::new(TokenType::Symbol, start, self.position),
                    Err((error_type, start)) => {
                        self.record_illegal_token(start, self.position, error_type)
                    }
                };
            }
            _ => {
                // Determine if it's an identifier or a number
                if self.ch.is_alphabetic() || self.ch == '_' {
                    let start = self.position;
                    let literal = self.read_identifier();

                    // Check for illegal identifiers starting with underscore followed by a digit
                    if literal.starts_with('_')
                        && literal
                            .trim_start_matches('_')
                            .chars()
                            .next()
                            .map_or(false, |c| c.is_numeric())
                    {
                        return self.record_illegal_token(
                            start,
                            self.position,
                            LexerErrorType::InvalidNumber,
                        );
                    }

                    // Check if the identifier is followed by a colon, indicating a symbol key
                    if self.ch == ':' {
                        self.read_ascii_char(); // Advance past the colon

                        return Token::new(TokenType::SymbolKey, start, self.position);
                    }

                    // Return the identifier which may also be a reserved keyword
                    return Token::new(lookup_identifier(&literal), start, self.position);
                } else if self.ch.is_ascii_digit() {
                    return match self.read_number() {
                        Ok((token_type, start)) => Token::new(token_type, start, self.position),
                        Err((error_type, start)) => {
                            return self.record_illegal_token(start, self.position, error_type);
                        }
                    };
                } else {
                    self.record_illegal_token(
                        self.position,
                        self.position + 1,
                        LexerErrorType::UnexpectedCharacter,
                    )
                }
            }
        };

        self.read_char();
        token
    }

    // -------------------------
    // === Character Reading ===
    // -------------------------

    /// Reads the next character and advances the lexer's positions.
    /// Handles both ASCII and Unicode characters.
    fn read_char(&mut self) {
        // If the previous character was a newline, move to the next line
        // before reading the new character. This ensures newlines are
        // tracked as the last character of their line, not the first of the next.
        if self.ch == '\n' {
            self.line += 1;
            self.column = 0;
        }

        if self.read_position >= self.input.len() {
            self.ch = '\0';
            self.position = self.read_position;
            return;
        }

        self.position = self.read_position;
        self.column += 1;

        let byte = self.input.as_bytes()[self.read_position];

        // ASCII bytes are always less than 128
        if byte < 128 {
            self.ch = byte as char;
            self.read_position += 1;
        } else {
            // Unicode: decode the character and advance by its byte length
            let ch = self.input[self.read_position..]
                .chars()
                .next()
                .unwrap_or('\0');
            self.ch = ch;
            self.read_position += ch.len_utf8();
        }
    }

    /// Reads the next ASCII character and advances the lexer's positions.
    /// Use this optimization when you know the next character is ASCII (operators, numbers).
    fn read_ascii_char(&mut self) {
        if self.ch == '\n' {
            self.line += 1;
            self.column = 0;
        }

        if self.read_position >= self.input.len() {
            self.ch = '\0';
            self.position = self.read_position;
            return;
        }

        self.ch = self.input.as_bytes()[self.read_position] as char;
        self.position = self.read_position;
        self.read_position += 1;
        self.column += 1;
    }

    /// Peeks at the next ASCII character without advancing the lexer's position.
    /// Returns the next character or '\0' if at the end of input.
    fn peek_ascii_char(&self) -> char {
        if self.read_position >= self.input.len() {
            '\0'
        } else {
            self.input.as_bytes()[self.read_position] as char
        }
    }

    /// Peeks at the next Unicode character without advancing the lexer's position.
    /// Returns the next character or '\0' if at the end of input.
    fn peek_unicode_char(&self) -> char {
        if self.read_position >= self.input.len() {
            '\0'
        } else {
            self.input[self.read_position..]
                .chars()
                .next()
                .unwrap_or('\0')
        }
    }

    // ---------------------
    // === Token Parsing ===
    // ---------------------

    /// Reads characters until a non-whitespace character is found.
    fn skip_whitespace(&mut self) {
        while Self::is_whitespace(self.ch) {
            self.read_char();
        }
    }

    /// Reads an identifier from the input.
    /// Returns the identifier as a &str slice.
    fn read_identifier(&mut self) -> &'a str {
        let position = self.position;

        while self.ch.is_alphanumeric() || self.ch == '_' {
            self.read_char();
        }

        if Self::is_suffix_char(self.ch) {
            self.read_char();
        }

        &self.input[position..self.position]
    }

    /// Reads a number (integer or float).
    /// Returns its TokenType and start position.
    fn read_number(&mut self) -> Result<(TokenType, usize), (LexerErrorType, usize)> {
        let start = self.position;

        // Read integer part and check for errors
        let integer_part_valid = self.read_number_part();

        // Check if we have a decimal point followed by a digit
        if self.ch == '.' && self.peek_ascii_char().is_ascii_digit() {
            self.read_ascii_char(); // consume '.'

            let decimal_part_valid = self.read_number_part();

            if integer_part_valid && decimal_part_valid {
                return Ok((TokenType::Float, start));
            } else {
                return Err((LexerErrorType::InvalidNumber, start));
            }
        }

        if integer_part_valid {
            Ok((TokenType::Integer, start))
        } else {
            Err((LexerErrorType::InvalidNumber, start))
        }
    }

    // Helper method to read a sequence of digits with optional underscores
    // Returns false if the sequence is invalid (consecutive underscores or trailing underscore)
    fn read_number_part(&mut self) -> bool {
        let mut last_was_underscore = false;
        let mut has_digits = false;

        while self.ch.is_ascii_digit() || self.ch == '_' {
            if self.ch == '_' {
                if last_was_underscore {
                    // Consecutive underscores - consume rest of number and mark as invalid
                    while self.ch.is_ascii_digit() || self.ch == '_' {
                        self.read_ascii_char();
                    }

                    return false;
                }

                last_was_underscore = true;
            } else {
                last_was_underscore = false;
                has_digits = true;
            }

            self.read_ascii_char();
        }

        // Number part ending with underscore is invalid
        !last_was_underscore && has_digits
    }

    /// Reads a string literal, handling escaped quotes.
    /// Returns the TokenType and the start position of the string.
    fn read_string(&mut self) -> Result<usize, (LexerErrorType, usize)> {
        let start = self.position;

        loop {
            self.read_char();

            // Handle escaped quotes and do not terminate the string prematurely
            if self.ch == '\\' && ESCAPABLE_CHARS.contains(&self.peek_unicode_char()) {
                self.read_char(); // Skip the escaped character
                continue;
            }

            // String ends with closing quote or unterminated at EOF
            if self.ch == '"' || self.ch == '\0' {
                break;
            }
        }

        if self.ch == '"' {
            self.read_char();

            Ok(start)
        } else {
            Err((LexerErrorType::UnterminatedString, start))
        }
    }

    /// Reads a symbol.
    /// Returns the TokenType and the start position of the symbol.
    fn read_symbol(&mut self) -> Result<usize, (LexerErrorType, usize)> {
        let start = self.position;

        self.read_char(); // Start position after the ':'

        if self.ch.is_alphabetic() || self.ch == '_' {
            // Read the symbol like an identifier
            self.read_identifier();

            Ok(start)
        } else if self.ch.is_ascii_digit()
            || (self.ch != '\0' && !self.ch.is_alphabetic() && !Self::is_whitespace(self.ch))
        // If the symbol starts with a number, or some special character
        {
            // Read until we hit an alphabetic character, whitespace, or EOF
            while self.ch != '\0' && !Self::is_whitespace(self.ch) && !self.ch.is_alphabetic() {
                self.read_char();
            }

            Err((LexerErrorType::InvalidSymbol, start))
        } else {
            Err((LexerErrorType::InvalidSymbol, start))
        }
    }

    /// Reads a comment until the end of the line or end of file.
    /// Returns the position after reading the comment.
    fn read_comment(&mut self) -> usize {
        while self.ch != '\n' && self.ch != '\0' {
            self.read_char();
        }

        self.position
    }

    /// Reads an operator, which may be a single or multi-character operator.
    /// Returns the position after reading the operator.
    fn read_operator(&mut self) -> usize {
        // Advance to the next character to complete the operator
        self.read_ascii_char();

        self.position + 1
    }

    // ----------------------
    // === Error Handling ===
    // ----------------------

    /// Takes a start position, end position, and error type to create an illegal token and record the error.
    /// Returns the illegal token.
    fn record_illegal_token(&mut self, start: usize, end: usize, error: LexerErrorType) -> Token {
        let token = Token::new(TokenType::Illegal, start, end);
        self.errors.push(LexerError::new(token, error));

        token
    }

    // -----------------
    // === Utilities ===
    // -----------------

    /// Determines if a character is a whitespace character.
    #[inline]
    fn is_whitespace(ch: char) -> bool {
        matches!(ch, ' ' | '\t' | '\n' | '\r')
    }

    #[inline]
    fn is_suffix_char(ch: char) -> bool {
        matches!(ch, '!' | '?')
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Token> {
        let token = self.next_token();

        if token.token_type == TokenType::EOF {
            None
        } else {
            Some(token)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Lexer, LexerErrorType};
    use crate::token::TokenType;

    fn assert_tokens(expected: Vec<(TokenType, &str)>, input: &str) {
        let tokens: Vec<_> = Lexer::new(input)
            .map(|t| (t.token_type, t.lexeme(input)))
            .collect();

        assert_eq!(
            expected.len(),
            tokens.len(),
            "Token count mismatch. Expected {}, got {}",
            expected.len(),
            tokens.len()
        );

        for (i, token) in tokens.iter().enumerate() {
            assert_eq!(expected[i], *token, "Token mismatch at index {}", i);
        }
    }

    mod operators {
        use super::*;

        #[test]
        fn binary_operators() {
            let input = "+ - * / ** => <= == >= != < > && || !";

            let expected = vec![
                (TokenType::Plus, "+"),
                (TokenType::Minus, "-"),
                (TokenType::Asterisk, "*"),
                (TokenType::Slash, "/"),
                (TokenType::Power, "**"),
                (TokenType::HashRocket, "=>"),
                (TokenType::LTE, "<="),
                (TokenType::Eq, "=="),
                (TokenType::GTE, ">="),
                (TokenType::NotEq, "!="),
                (TokenType::LT, "<"),
                (TokenType::GT, ">"),
                (TokenType::And, "&&"),
                (TokenType::Or, "||"),
                (TokenType::Bang, "!"),
            ];

            assert_tokens(expected, input);
        }

        #[test]
        fn unary_operators() {
            let input = "!true -4";

            let expected = vec![
                (TokenType::Bang, "!"),
                (TokenType::True, "true"),
                (TokenType::Minus, "-"),
                (TokenType::Integer, "4"),
            ];

            assert_tokens(expected, input);
        }

        #[test]
        fn plus_variations() {
            let input = "+ ++ +a a+";

            let expected = vec![
                (TokenType::Plus, "+"),
                (TokenType::Plus, "+"),
                (TokenType::Plus, "+"),
                (TokenType::Plus, "+"),
                (TokenType::Identifier, "a"),
                (TokenType::Identifier, "a"),
                (TokenType::Plus, "+"),
            ];

            assert_tokens(expected, input);
        }

        #[test]
        fn minus_variations() {
            let input = "- -- -a a-";

            let expected = vec![
                (TokenType::Minus, "-"),
                (TokenType::Minus, "-"),
                (TokenType::Minus, "-"),
                (TokenType::Minus, "-"),
                (TokenType::Identifier, "a"),
                (TokenType::Identifier, "a"),
                (TokenType::Minus, "-"),
            ];

            assert_tokens(expected, input);
        }

        #[test]
        fn asterisk_variations() {
            let input = "* ** *a a*";

            let expected = vec![
                (TokenType::Asterisk, "*"),
                (TokenType::Power, "**"),
                (TokenType::Asterisk, "*"),
                (TokenType::Identifier, "a"),
                (TokenType::Identifier, "a"),
                (TokenType::Asterisk, "*"),
            ];

            assert_tokens(expected, input);
        }

        #[test]
        fn slash_variations() {
            let input = "/ // /a a/";

            let expected = vec![
                (TokenType::Slash, "/"),
                (TokenType::Slash, "/"),
                (TokenType::Slash, "/"),
                (TokenType::Slash, "/"),
                (TokenType::Identifier, "a"),
                (TokenType::Identifier, "a"),
                (TokenType::Slash, "/"),
            ];

            assert_tokens(expected, input);
        }

        #[test]
        fn power_variations() {
            let input = "** *** **a a**";

            let expected = vec![
                (TokenType::Power, "**"),
                (TokenType::Power, "**"),
                (TokenType::Asterisk, "*"),
                (TokenType::Power, "**"),
                (TokenType::Identifier, "a"),
                (TokenType::Identifier, "a"),
                (TokenType::Power, "**"),
            ];

            assert_tokens(expected, input);
        }

        #[test]
        fn hash_rocket_variations() {
            let input = "=> ==> =>= =>> =>< <=> =>a a=>";

            let expected = vec![
                (TokenType::HashRocket, "=>"),
                (TokenType::Eq, "=="),
                (TokenType::GT, ">"),
                (TokenType::HashRocket, "=>"),
                (TokenType::Assign, "="),
                (TokenType::HashRocket, "=>"),
                (TokenType::GT, ">"),
                (TokenType::HashRocket, "=>"),
                (TokenType::LT, "<"),
                (TokenType::LTE, "<="),
                (TokenType::GT, ">"),
                (TokenType::HashRocket, "=>"),
                (TokenType::Identifier, "a"),
                (TokenType::Identifier, "a"),
                (TokenType::HashRocket, "=>"),
            ];

            assert_tokens(expected, input);
        }

        #[test]
        fn lte_variations() {
            let input = "<= <<= <== =<= <<= ><= <=a a<=";

            let expected = vec![
                (TokenType::LTE, "<="),
                (TokenType::LT, "<"),
                (TokenType::LTE, "<="),
                (TokenType::LTE, "<="),
                (TokenType::Assign, "="),
                (TokenType::Assign, "="),
                (TokenType::LTE, "<="),
                (TokenType::LT, "<"),
                (TokenType::LTE, "<="),
                (TokenType::GT, ">"),
                (TokenType::LTE, "<="),
                (TokenType::LTE, "<="),
                (TokenType::Identifier, "a"),
                (TokenType::Identifier, "a"),
                (TokenType::LTE, "<="),
            ];

            assert_tokens(expected, input);
        }

        #[test]
        fn equality_variations() {
            let input = "= == === =a ==a a==";

            let expected = vec![
                (TokenType::Assign, "="),
                (TokenType::Eq, "=="),
                (TokenType::Eq, "=="),
                (TokenType::Assign, "="),
                (TokenType::Assign, "="),
                (TokenType::Identifier, "a"),
                (TokenType::Eq, "=="),
                (TokenType::Identifier, "a"),
                (TokenType::Identifier, "a"),
                (TokenType::Eq, "=="),
            ];

            assert_tokens(expected, input);
        }

        #[test]
        fn gte_variations() {
            let input = ">= >== =>= >=< >>= >=a a>=";

            let expected = vec![
                (TokenType::GTE, ">="),
                (TokenType::GTE, ">="),
                (TokenType::Assign, "="),
                (TokenType::HashRocket, "=>"),
                (TokenType::Assign, "="),
                (TokenType::GTE, ">="),
                (TokenType::LT, "<"),
                (TokenType::GT, ">"),
                (TokenType::GTE, ">="),
                (TokenType::GTE, ">="),
                (TokenType::Identifier, "a"),
                (TokenType::Identifier, "a"),
                (TokenType::GTE, ">="),
            ];

            assert_tokens(expected, input);
        }

        #[test]
        fn lt_and_gt_variations() {
            let input = "< << >< <> > >> <a a> <a a<";

            let expected = vec![
                (TokenType::LT, "<"),
                (TokenType::LT, "<"),
                (TokenType::LT, "<"),
                (TokenType::GT, ">"),
                (TokenType::LT, "<"),
                (TokenType::LT, "<"),
                (TokenType::GT, ">"),
                (TokenType::GT, ">"),
                (TokenType::GT, ">"),
                (TokenType::GT, ">"),
                (TokenType::LT, "<"),
                (TokenType::Identifier, "a"),
                (TokenType::Identifier, "a"),
                (TokenType::GT, ">"),
                (TokenType::LT, "<"),
                (TokenType::Identifier, "a"),
                (TokenType::Identifier, "a"),
                (TokenType::LT, "<"),
            ];

            assert_tokens(expected, input);
        }

        #[test]
        fn and_variations() {
            let input = "&& &&& &&&& &&a a&&";

            let expected = vec![
                (TokenType::And, "&&"),
                (TokenType::And, "&&"),
                (TokenType::Illegal, "&"),
                (TokenType::And, "&&"),
                (TokenType::And, "&&"),
                (TokenType::And, "&&"),
                (TokenType::Identifier, "a"),
                (TokenType::Identifier, "a"),
                (TokenType::And, "&&"),
            ];

            assert_tokens(expected, input);
        }

        #[test]
        fn or_variations() {
            let input = "|| ||| |||| ||a a||";

            let expected = vec![
                (TokenType::Or, "||"),
                (TokenType::Or, "||"),
                (TokenType::Illegal, "|"),
                (TokenType::Or, "||"),
                (TokenType::Or, "||"),
                (TokenType::Or, "||"),
                (TokenType::Identifier, "a"),
                (TokenType::Identifier, "a"),
                (TokenType::Or, "||"),
            ];

            assert_tokens(expected, input);
        }

        #[test]
        fn not_variations() {
            let input = "! != !! !a a!";

            let expected = vec![
                (TokenType::Bang, "!"),
                (TokenType::NotEq, "!="),
                (TokenType::Bang, "!"),
                (TokenType::Bang, "!"),
                (TokenType::Bang, "!"),
                (TokenType::Identifier, "a"),
                (TokenType::Identifier, "a!"),
            ];

            assert_tokens(expected, input);
        }

        #[test]
        fn special_characters() {
            let input = "; { } [ ] , . | a; ;a a{ {a a} }a a[ [a a] ]a a, ,a a. .a";

            let expected = vec![
                (TokenType::Semicolon, ";"),
                (TokenType::LBrace, "{"),
                (TokenType::RBrace, "}"),
                (TokenType::LBrack, "["),
                (TokenType::RBrack, "]"),
                (TokenType::Comma, ","),
                (TokenType::Dot, "."),
                (TokenType::Illegal, "|"),
                (TokenType::Identifier, "a"),
                (TokenType::Semicolon, ";"),
                (TokenType::Semicolon, ";"),
                (TokenType::Identifier, "a"),
                (TokenType::Identifier, "a"),
                (TokenType::LBrace, "{"),
                (TokenType::LBrace, "{"),
                (TokenType::Identifier, "a"),
                (TokenType::Identifier, "a"),
                (TokenType::RBrace, "}"),
                (TokenType::RBrace, "}"),
                (TokenType::Identifier, "a"),
                (TokenType::Identifier, "a"),
                (TokenType::LBrack, "["),
                (TokenType::LBrack, "["),
                (TokenType::Identifier, "a"),
                (TokenType::Identifier, "a"),
                (TokenType::RBrack, "]"),
                (TokenType::RBrack, "]"),
                (TokenType::Identifier, "a"),
                (TokenType::Identifier, "a"),
                (TokenType::Comma, ","),
                (TokenType::Comma, ","),
                (TokenType::Identifier, "a"),
                (TokenType::Identifier, "a"),
                (TokenType::Dot, "."),
                (TokenType::Dot, "."),
                (TokenType::Identifier, "a"),
            ];

            assert_tokens(expected, input);
        }
    }

    mod literals {
        use super::*;

        #[test]
        fn keywords() {
            let input = "def do end if else elsif while return true false nil DEF DO END IF ELSE ELSIF WHILE RETURN TRUE FALSE NIL";

            let expected = vec![
                (TokenType::Def, "def"),
                (TokenType::Do, "do"),
                (TokenType::End, "end"),
                (TokenType::If, "if"),
                (TokenType::Else, "else"),
                (TokenType::Elsif, "elsif"),
                (TokenType::While, "while"),
                (TokenType::Return, "return"),
                (TokenType::True, "true"),
                (TokenType::False, "false"),
                (TokenType::Nil, "nil"),
                (TokenType::Identifier, "DEF"),
                (TokenType::Identifier, "DO"),
                (TokenType::Identifier, "END"),
                (TokenType::Identifier, "IF"),
                (TokenType::Identifier, "ELSE"),
                (TokenType::Identifier, "ELSIF"),
                (TokenType::Identifier, "WHILE"),
                (TokenType::Identifier, "RETURN"),
                (TokenType::Identifier, "TRUE"),
                (TokenType::Identifier, "FALSE"),
                (TokenType::Identifier, "NIL"),
            ];

            assert_tokens(expected, input);
        }

        #[test]
        fn keywords_with_suffixes() {
            let input = "defx endian donut iffy elsify truex falsenil nilly";

            let expected = vec![
                (TokenType::Identifier, "defx"),
                (TokenType::Identifier, "endian"),
                (TokenType::Identifier, "donut"),
                (TokenType::Identifier, "iffy"),
                (TokenType::Identifier, "elsify"),
                (TokenType::Identifier, "truex"),
                (TokenType::Identifier, "falsenil"),
                (TokenType::Identifier, "nilly"),
            ];

            assert_tokens(expected, input);
        }

        #[test]
        fn identifiers() {
            let input = "_ _test test_ __private__ test! test1 1test test? !test test_1 test_! test_? 1_test !_test foo.bar.baz Hello_世界 café";

            let expected = vec![
                (TokenType::Identifier, "_"),
                (TokenType::Identifier, "_test"),
                (TokenType::Identifier, "test_"),
                (TokenType::Identifier, "__private__"),
                (TokenType::Identifier, "test!"),
                (TokenType::Identifier, "test1"),
                (TokenType::Integer, "1"),
                (TokenType::Identifier, "test"),
                (TokenType::Identifier, "test?"),
                (TokenType::Bang, "!"),
                (TokenType::Identifier, "test"),
                (TokenType::Identifier, "test_1"),
                (TokenType::Identifier, "test_!"),
                (TokenType::Identifier, "test_?"),
                (TokenType::Illegal, "1_"),
                (TokenType::Identifier, "test"),
                (TokenType::Bang, "!"),
                (TokenType::Identifier, "_test"),
                (TokenType::Identifier, "foo"),
                (TokenType::Dot, "."),
                (TokenType::Identifier, "bar"),
                (TokenType::Dot, "."),
                (TokenType::Identifier, "baz"),
                (TokenType::Identifier, "Hello_世界"),
                (TokenType::Identifier, "café"),
            ];

            assert_tokens(expected, input);
        }

        #[test]
        fn symbols() {
            let input = ": :_ :_test :test_ :__private__ :test! :test1 :1test :test? :!test :test_1 :test_! :test_? :1_test :!_test :!!_test";

            let expected = vec![
                (TokenType::Illegal, ":"),
                (TokenType::Symbol, ":_"),
                (TokenType::Symbol, ":_test"),
                (TokenType::Symbol, ":test_"),
                (TokenType::Symbol, ":__private__"),
                (TokenType::Symbol, ":test!"),
                (TokenType::Symbol, ":test1"),
                (TokenType::Illegal, ":1"),
                (TokenType::Identifier, "test"),
                (TokenType::Symbol, ":test?"),
                (TokenType::Illegal, ":!"),
                (TokenType::Identifier, "test"),
                (TokenType::Symbol, ":test_1"),
                (TokenType::Symbol, ":test_!"),
                (TokenType::Symbol, ":test_?"),
                (TokenType::Illegal, ":1_"),
                (TokenType::Identifier, "test"),
                (TokenType::Illegal, ":!_"),
                (TokenType::Identifier, "test"),
                (TokenType::Illegal, ":!!_"),
                (TokenType::Identifier, "test"),
            ];

            assert_tokens(expected, input);
        }

        #[test]
        fn literals() {
            let input =
                "0 42 3.14 0.0 -5 -5.12 01 00 001.2 1_2_3_4 1_2_3_4.1 1.1_2_3 1_ 1._2 1_.2 1.2.3";

            let expected = vec![
                (TokenType::Integer, "0"),
                (TokenType::Integer, "42"),
                (TokenType::Float, "3.14"),
                (TokenType::Float, "0.0"),
                (TokenType::Minus, "-"),
                (TokenType::Integer, "5"),
                (TokenType::Minus, "-"),
                (TokenType::Float, "5.12"),
                (TokenType::Integer, "01"),
                (TokenType::Integer, "00"),
                (TokenType::Float, "001.2"),
                (TokenType::Integer, "1_2_3_4"),
                (TokenType::Float, "1_2_3_4.1"),
                (TokenType::Float, "1.1_2_3"),
                (TokenType::Illegal, "1_"),
                (TokenType::Integer, "1"),
                (TokenType::Dot, "."),
                (TokenType::Illegal, "_2"),
                (TokenType::Illegal, "1_.2"),
                (TokenType::Float, "1.2"),
                (TokenType::Dot, "."),
                (TokenType::Integer, "3"),
            ];

            assert_tokens(expected, input);
        }

        #[test]
        fn strings() {
            let input = r#""hello" "hello world" "h\"ello" "5.7" "Hello 世界" "café" ""#;

            let expected = vec![
                (TokenType::String, "\"hello\""),
                (TokenType::String, "\"hello world\""),
                (TokenType::String, "\"h\\\"ello\""),
                (TokenType::String, "\"5.7\""),
                (TokenType::String, "\"Hello 世界\""),
                (TokenType::String, "\"café\""),
                (TokenType::Illegal, "\""),
            ];

            assert_tokens(expected, input);
        }

        #[test]
        fn single_quote() {
            let input = r#"""#;

            let expected = vec![(TokenType::Illegal, "\"")];

            assert_tokens(expected, input);
        }

        #[test]
        fn comments() {
            let input = r#"# This is a comment
            # This is a comment with a # inside
            #Another comment
            #    Indented comment
        #"#;

            let expected = vec![
                (TokenType::Comment, "# This is a comment"),
                (TokenType::Comment, "# This is a comment with a # inside"),
                (TokenType::Comment, "#Another comment"),
                (TokenType::Comment, "#    Indented comment"),
                (TokenType::Comment, "#"),
            ];

            assert_tokens(expected, input);
        }
    }

    mod whitespace {
        use super::*;

        #[test]
        fn empty_input() {
            let input = "";

            assert_eq!(0, Lexer::new(input).count());
        }

        #[test]
        fn whitespace_only() {
            let input = "     ";

            assert_eq!(0, Lexer::new(input).count());

            let input = "    \n\t   \r\n  ";

            assert_eq!(0, Lexer::new(input).count());
        }

        #[test]
        fn whitespace_variations() {
            let input = "x=5+3 x    =    5 x=\n5 x=\r\n5";

            let expected = vec![
                (TokenType::Identifier, "x"),
                (TokenType::Assign, "="),
                (TokenType::Integer, "5"),
                (TokenType::Plus, "+"),
                (TokenType::Integer, "3"),
                (TokenType::Identifier, "x"),
                (TokenType::Assign, "="),
                (TokenType::Integer, "5"),
                (TokenType::Identifier, "x"),
                (TokenType::Assign, "="),
                (TokenType::Integer, "5"),
                (TokenType::Identifier, "x"),
                (TokenType::Assign, "="),
                (TokenType::Integer, "5"),
            ];

            assert_tokens(expected, input);
        }
    }

    mod integration {
        use super::*;

        #[test]
        fn token_recognition() {
            let input = r##"
            # Types
            1
            4.0
            :my_symbol
            "This is a string"

            # Binary Operators
            1 + 2 * (3 - 4) / 5
            2 ** 3
            x == y
            x != y
            x >= 10
            x <= 10
            x < 5
            x > 2

            # Functions
            def hello_world(name) do
                return puts "Hello, #{name}!"
            end

            object.method_name(1)

            # Control Flow
            if x > 5 do
                result = x + y
            elsif y < 10 do
                result = x - y
            else do
                nil
            end

            while x < 10 do
                x = x + 1
            end

            # Boolean logic
            true && false
            true || false
            !true

            # Hashes
            { name: "John", age => 30, "height" => 5.9 }
        "##;

            let expected = vec![
                (TokenType::Comment, "# Types"),
                (TokenType::Integer, "1"),
                (TokenType::Float, "4.0"),
                (TokenType::Symbol, ":my_symbol"),
                (TokenType::String, "\"This is a string\""),
                (TokenType::Comment, "# Binary Operators"),
                (TokenType::Integer, "1"),
                (TokenType::Plus, "+"),
                (TokenType::Integer, "2"),
                (TokenType::Asterisk, "*"),
                (TokenType::LParen, "("),
                (TokenType::Integer, "3"),
                (TokenType::Minus, "-"),
                (TokenType::Integer, "4"),
                (TokenType::RParen, ")"),
                (TokenType::Slash, "/"),
                (TokenType::Integer, "5"),
                (TokenType::Integer, "2"),
                (TokenType::Power, "**"),
                (TokenType::Integer, "3"),
                (TokenType::Identifier, "x"),
                (TokenType::Eq, "=="),
                (TokenType::Identifier, "y"),
                (TokenType::Identifier, "x"),
                (TokenType::NotEq, "!="),
                (TokenType::Identifier, "y"),
                (TokenType::Identifier, "x"),
                (TokenType::GTE, ">="),
                (TokenType::Integer, "10"),
                (TokenType::Identifier, "x"),
                (TokenType::LTE, "<="),
                (TokenType::Integer, "10"),
                (TokenType::Identifier, "x"),
                (TokenType::LT, "<"),
                (TokenType::Integer, "5"),
                (TokenType::Identifier, "x"),
                (TokenType::GT, ">"),
                (TokenType::Integer, "2"),
                (TokenType::Comment, "# Functions"),
                (TokenType::Def, "def"),
                (TokenType::Identifier, "hello_world"),
                (TokenType::LParen, "("),
                (TokenType::Identifier, "name"),
                (TokenType::RParen, ")"),
                (TokenType::Do, "do"),
                (TokenType::Return, "return"),
                (TokenType::Identifier, "puts"),
                (TokenType::String, "\"Hello, #{name}!\""),
                (TokenType::End, "end"),
                (TokenType::Identifier, "object"),
                (TokenType::Dot, "."),
                (TokenType::Identifier, "method_name"),
                (TokenType::LParen, "("),
                (TokenType::Integer, "1"),
                (TokenType::RParen, ")"),
                (TokenType::Comment, "# Control Flow"),
                (TokenType::If, "if"),
                (TokenType::Identifier, "x"),
                (TokenType::GT, ">"),
                (TokenType::Integer, "5"),
                (TokenType::Do, "do"),
                (TokenType::Identifier, "result"),
                (TokenType::Assign, "="),
                (TokenType::Identifier, "x"),
                (TokenType::Plus, "+"),
                (TokenType::Identifier, "y"),
                (TokenType::Elsif, "elsif"),
                (TokenType::Identifier, "y"),
                (TokenType::LT, "<"),
                (TokenType::Integer, "10"),
                (TokenType::Do, "do"),
                (TokenType::Identifier, "result"),
                (TokenType::Assign, "="),
                (TokenType::Identifier, "x"),
                (TokenType::Minus, "-"),
                (TokenType::Identifier, "y"),
                (TokenType::Else, "else"),
                (TokenType::Do, "do"),
                (TokenType::Nil, "nil"),
                (TokenType::End, "end"),
                (TokenType::While, "while"),
                (TokenType::Identifier, "x"),
                (TokenType::LT, "<"),
                (TokenType::Integer, "10"),
                (TokenType::Do, "do"),
                (TokenType::Identifier, "x"),
                (TokenType::Assign, "="),
                (TokenType::Identifier, "x"),
                (TokenType::Plus, "+"),
                (TokenType::Integer, "1"),
                (TokenType::End, "end"),
                (TokenType::Comment, "# Boolean logic"),
                (TokenType::True, "true"),
                (TokenType::And, "&&"),
                (TokenType::False, "false"),
                (TokenType::True, "true"),
                (TokenType::Or, "||"),
                (TokenType::False, "false"),
                (TokenType::Bang, "!"),
                (TokenType::True, "true"),
                (TokenType::Comment, "# Hashes"),
                (TokenType::LBrace, "{"),
                (TokenType::SymbolKey, "name:"),
                (TokenType::String, "\"John\""),
                (TokenType::Comma, ","),
                (TokenType::Identifier, "age"),
                (TokenType::HashRocket, "=>"),
                (TokenType::Integer, "30"),
                (TokenType::Comma, ","),
                (TokenType::String, "\"height\""),
                (TokenType::HashRocket, "=>"),
                (TokenType::Float, "5.9"),
                (TokenType::RBrace, "}"),
            ];

            assert_tokens(expected, input);
        }
    }

    mod position_tracking {
        use super::*;

        #[test]
        fn single_line() {
            let input = "abc";
            let mut lexer = Lexer::new(input);

            // After initialization, we've read 'a', so column is 1
            assert_eq!(lexer.line(), 1);
            assert_eq!(lexer.column(), 1);

            lexer.next_token(); // consumes "abc", stops at EOF

            assert_eq!(lexer.line(), 1);
            assert_eq!(lexer.column(), 3); // EOF doesn't increment column
        }

        #[test]
        fn newlines() {
            let input = "a\nb\nc";
            let mut lexer = Lexer::new(input);

            assert_eq!(lexer.line(), 1);
            assert_eq!(lexer.column(), 1);

            // After reading "a", lexer advances to '\n' which stays on line 1
            // (newline is the last char of the current line, not first of the next)
            lexer.next_token(); // "a"
            assert_eq!(lexer.line(), 1);
            assert_eq!(lexer.column(), 2); // '\n' is at line 1, column 2

            // skip_whitespace reads past '\n' (incrementing line), then "b" advances to next '\n'
            lexer.next_token(); // "b"
            assert_eq!(lexer.line(), 2);
            assert_eq!(lexer.column(), 2); // '\n' is at line 2, column 2

            // skip_whitespace reads past '\n' (incrementing line), then "c" advances to EOF
            lexer.next_token(); // "c"
            assert_eq!(lexer.line(), 3);
            assert_eq!(lexer.column(), 1); // EOF doesn't increment
        }

        #[test]
        fn unicode() {
            let input = "Hello_世界";
            let mut lexer = Lexer::new(input);

            assert_eq!(lexer.line(), 1);
            assert_eq!(lexer.column(), 1);

            lexer.next_token(); // "Hello_世界"

            // Column tracks character count, not byte count
            // "Hello_世界" is 8 characters, EOF doesn't increment
            assert_eq!(lexer.line(), 1);
            assert_eq!(lexer.column(), 8);
        }

        #[test]
        fn unicode_multiline() {
            let input = "café\n世界";
            let mut lexer = Lexer::new(input);

            // "café" is 4 chars, then advances to '\n' which stays on line 1
            lexer.next_token(); // "café"
            assert_eq!(lexer.line(), 1);
            assert_eq!(lexer.column(), 5); // '\n' is at line 1, column 5

            // skip_whitespace moves past '\n' (incrementing line), reads "世界" (2 chars), stops at EOF
            lexer.next_token(); // "世界"
            assert_eq!(lexer.line(), 2);
            assert_eq!(lexer.column(), 2);
        }

        #[test]
        fn with_operators() {
            let input = "x == y";
            let mut lexer = Lexer::new(input);

            // "x" read, advances to ' ' (column=2)
            lexer.next_token(); // "x"
            assert_eq!(lexer.column(), 2);

            // skip_whitespace to '=', read "==", advances past (column=5)
            lexer.next_token(); // "=="
            assert_eq!(lexer.column(), 5);

            // skip_whitespace to 'y', read "y", advances to EOF (column=6, no increment)
            lexer.next_token(); // "y"
            assert_eq!(lexer.column(), 6);
        }

        #[test]
        fn with_string() {
            let input = "\"hello\"";
            let mut lexer = Lexer::new(input);

            // String is 7 chars, advances to EOF (no increment)
            lexer.next_token(); // "\"hello\""
            assert_eq!(lexer.line(), 1);
            assert_eq!(lexer.column(), 7);
        }
    }

    mod error_handling {
        use super::*;

        #[test]
        fn unexpected_character_single_ampersand() {
            let input = "&";
            let mut lexer = Lexer::new(input);

            let token = lexer.next_token();
            assert_eq!(token.token_type, TokenType::Illegal);
            assert_eq!(token.lexeme(input), "&");

            let errors = lexer.errors();
            assert_eq!(errors.len(), 1);
            assert!(matches!(
                errors[0].error,
                LexerErrorType::UnexpectedCharacter
            ));
            assert_eq!(errors[0].message(input), "Unexpected character: \"&\"");
        }

        #[test]
        fn unexpected_character_single_pipe() {
            let input = "|";
            let mut lexer = Lexer::new(input);

            let token = lexer.next_token();
            assert_eq!(token.token_type, TokenType::Illegal);
            assert_eq!(token.lexeme(input), "|");

            let errors = lexer.errors();
            assert_eq!(errors.len(), 1);
            assert!(matches!(
                errors[0].error,
                LexerErrorType::UnexpectedCharacter
            ));
        }

        #[test]
        fn unterminated_string() {
            let input = "\"hello";
            let mut lexer = Lexer::new(input);

            let token = lexer.next_token();
            assert_eq!(token.token_type, TokenType::Illegal);
            assert_eq!(token.lexeme(input), "\"hello");

            let errors = lexer.errors();
            assert_eq!(errors.len(), 1);
            assert!(matches!(
                errors[0].error,
                LexerErrorType::UnterminatedString
            ));
            assert_eq!(
                errors[0].message(input),
                "Unterminated string: \"\\\"hello\""
            );
        }

        #[test]
        fn unterminated_string_empty() {
            let input = "\"";
            let mut lexer = Lexer::new(input);

            let token = lexer.next_token();
            assert_eq!(token.token_type, TokenType::Illegal);

            let errors = lexer.errors();
            assert_eq!(errors.len(), 1);
            assert!(matches!(
                errors[0].error,
                LexerErrorType::UnterminatedString
            ));
        }

        #[test]
        fn invalid_number_trailing_underscore() {
            let input = "123_";
            let mut lexer = Lexer::new(input);

            let token = lexer.next_token();
            assert_eq!(token.token_type, TokenType::Illegal);
            assert_eq!(token.lexeme(input), "123_");

            let errors = lexer.errors();
            assert_eq!(errors.len(), 1);
            assert!(matches!(errors[0].error, LexerErrorType::InvalidNumber));
            assert_eq!(errors[0].message(input), "Invalid number format: \"123_\"");
        }

        #[test]
        fn invalid_number_consecutive_underscores() {
            let input = "1__2";
            let mut lexer = Lexer::new(input);

            let token = lexer.next_token();
            assert_eq!(token.token_type, TokenType::Illegal);
            assert_eq!(token.lexeme(input), "1__2");

            let errors = lexer.errors();
            assert_eq!(errors.len(), 1);
            assert!(matches!(errors[0].error, LexerErrorType::InvalidNumber));
        }

        #[test]
        fn invalid_number_underscore_prefix_digit() {
            // Identifiers starting with underscore followed by digit are invalid
            let input = "_1";
            let mut lexer = Lexer::new(input);

            let token = lexer.next_token();
            assert_eq!(token.token_type, TokenType::Illegal);
            assert_eq!(token.lexeme(input), "_1");

            let errors = lexer.errors();
            assert_eq!(errors.len(), 1);
            assert!(matches!(errors[0].error, LexerErrorType::InvalidNumber));
        }

        #[test]
        fn invalid_symbol_starts_with_digit() {
            let input = ":1test";
            let mut lexer = Lexer::new(input);

            let token = lexer.next_token();
            assert_eq!(token.token_type, TokenType::Illegal);

            let errors = lexer.errors();
            assert_eq!(errors.len(), 1);
            assert!(matches!(errors[0].error, LexerErrorType::InvalidSymbol));
        }

        #[test]
        fn invalid_symbol_starts_with_bang() {
            let input = ":!test";
            let mut lexer = Lexer::new(input);

            let token = lexer.next_token();
            assert_eq!(token.token_type, TokenType::Illegal);

            let errors = lexer.errors();
            assert_eq!(errors.len(), 1);
            assert!(matches!(errors[0].error, LexerErrorType::InvalidSymbol));
        }

        #[test]
        fn invalid_symbol_empty() {
            let input = ": ";
            let mut lexer = Lexer::new(input);

            let token = lexer.next_token();
            assert_eq!(token.token_type, TokenType::Illegal);

            let errors = lexer.errors();
            assert_eq!(errors.len(), 1);
            assert!(matches!(errors[0].error, LexerErrorType::InvalidSymbol));
        }

        #[test]
        fn multiple_errors_in_input() {
            let input = "& | 1_ :!";
            let mut lexer = Lexer::new(input);

            // Consume all tokens
            let tokens: Vec<_> = lexer.by_ref().collect();

            // Should have 4 illegal tokens
            assert_eq!(
                tokens
                    .iter()
                    .filter(|t| t.token_type == TokenType::Illegal)
                    .count(),
                4
            );

            // Should have 4 errors recorded
            let errors = lexer.errors();
            assert_eq!(errors.len(), 4);

            assert!(matches!(
                errors[0].error,
                LexerErrorType::UnexpectedCharacter
            )); // &
            assert!(matches!(
                errors[1].error,
                LexerErrorType::UnexpectedCharacter
            )); // |
            assert!(matches!(errors[2].error, LexerErrorType::InvalidNumber)); // 1_
            assert!(matches!(errors[3].error, LexerErrorType::InvalidSymbol)); // :!
        }

        #[test]
        fn mixed_valid_and_invalid_tokens() {
            let input = "def x = 1_ + 2";
            let mut lexer = Lexer::new(input);

            let _tokens: Vec<_> = lexer.by_ref().collect();

            let expected = vec![
                (TokenType::Def, "def"),
                (TokenType::Identifier, "x"),
                (TokenType::Assign, "="),
                (TokenType::Illegal, "1_"),
                (TokenType::Plus, "+"),
                (TokenType::Integer, "2"),
            ];

            assert_tokens(expected, input);

            let errors = lexer.errors();
            assert_eq!(errors.len(), 1);
            assert!(matches!(errors[0].error, LexerErrorType::InvalidNumber));
        }

        #[test]
        fn continues_lexing() {
            let input = "& valid";
            let mut lexer = Lexer::new(input);

            let token1 = lexer.next_token();
            assert_eq!(token1.token_type, TokenType::Illegal);

            let token2 = lexer.next_token();
            assert_eq!(token2.token_type, TokenType::Identifier);
            assert_eq!(token2.lexeme(input), "valid");

            assert_eq!(lexer.errors().len(), 1);
        }
    }
}
