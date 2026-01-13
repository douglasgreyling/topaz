use crate::token::{Token, TokenType, lookup_identifier};

const WHITESPACE_CHARS: [char; 4] = [' ', '\t', '\n', '\r'];
const SUFFIX_CHARS: [char; 2] = ['!', '?'];

pub struct Lexer<'a> {
    input: &'a str,
    position: usize,
    read_position: usize,
    ch: char,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        let mut lexer = Lexer {
            input,
            position: 0,
            read_position: 0,
            ch: '\0',
        };

        lexer.read_ascii_char();
        lexer
    }

    pub fn next_token(&mut self) -> Token<'a> {
        self.skip_whitespace();

        let token = match self.ch {
            '\0' => Token::new(TokenType::EOF, ""),
            '(' => Token::new(
                TokenType::LParen,
                &self.input[self.position..self.position + 1],
            ),
            ')' => Token::new(
                TokenType::RParen,
                &self.input[self.position..self.position + 1],
            ),
            '{' => Token::new(
                TokenType::LBrace,
                &self.input[self.position..self.position + 1],
            ),
            '}' => Token::new(
                TokenType::RBrace,
                &self.input[self.position..self.position + 1],
            ),
            '[' => Token::new(
                TokenType::LBrack,
                &self.input[self.position..self.position + 1],
            ),
            ']' => Token::new(
                TokenType::RBrack,
                &self.input[self.position..self.position + 1],
            ),
            ',' => Token::new(
                TokenType::Comma,
                &self.input[self.position..self.position + 1],
            ),
            ';' => Token::new(
                TokenType::Semicolon,
                &self.input[self.position..self.position + 1],
            ),
            '.' => Token::new(
                TokenType::Dot,
                &self.input[self.position..self.position + 1],
            ),
            '+' => Token::new(
                TokenType::Plus,
                &self.input[self.position..self.position + 1],
            ),
            '-' => Token::new(
                TokenType::Minus,
                &self.input[self.position..self.position + 1],
            ),
            '/' => Token::new(
                TokenType::Slash,
                &self.input[self.position..self.position + 1],
            ),
            '#' => Token::new(TokenType::Comment, self.read_comment()),
            '=' => match self.peek_ascii_char() {
                '=' => Token::new(TokenType::Eq, self.read_operator()),
                '>' => Token::new(TokenType::HashRocket, self.read_operator()),
                _ => Token::new(
                    TokenType::Assign,
                    &self.input[self.position..self.position + 1],
                ),
            },
            '!' => match self.peek_ascii_char() {
                '=' => Token::new(TokenType::NotEq, self.read_operator()),
                _ => Token::new(
                    TokenType::Bang,
                    &self.input[self.position..self.position + 1],
                ),
            },
            '<' => match self.peek_ascii_char() {
                '=' => Token::new(TokenType::LTE, self.read_operator()),
                _ => Token::new(TokenType::LT, &self.input[self.position..self.position + 1]),
            },
            '>' => match self.peek_ascii_char() {
                '=' => Token::new(TokenType::GTE, self.read_operator()),
                _ => Token::new(TokenType::GT, &self.input[self.position..self.position + 1]),
            },
            '*' => match self.peek_ascii_char() {
                '*' => Token::new(TokenType::Power, self.read_operator()),
                _ => Token::new(
                    TokenType::Asterisk,
                    &self.input[self.position..self.position + 1],
                ),
            },
            '&' => match self.peek_ascii_char() {
                '&' => Token::new(TokenType::And, self.read_operator()),
                _ => Token::new(
                    TokenType::Illegal,
                    &self.input[self.position..self.position + 1],
                ),
            },
            '|' => match self.peek_ascii_char() {
                '|' => Token::new(TokenType::Or, self.read_operator()),
                _ => Token::new(
                    TokenType::Illegal,
                    &self.input[self.position..self.position + 1],
                ),
            },
            '"' => {
                let (token_type, literal) = self.read_string();

                return Token::new(token_type, literal);
            }
            ':' => {
                let (token_type, literal) = self.read_symbol();

                return Token::new(token_type, literal);
            }
            _ => {
                // Determine if it's an identifier or a number
                if self.ch.is_alphabetic() || self.ch == '_' {
                    let start = self.position;
                    let literal = self.read_identifier();

                    // Check for illegal identifiers starting with underscore followed by a digit
                    if literal.starts_with('_') {
                        if let Some(first_non_underscore) =
                            literal.chars().skip_while(|&c| c == '_').next()
                        {
                            if first_non_underscore.is_numeric() {
                                return Token::new(TokenType::Illegal, literal);
                            }
                        }
                    }

                    // Check if the identifier is followed by a colon, indicating a symbol key
                    if self.ch == ':' {
                        self.read_ascii_char(); // Advance past the colon

                        return Token::new(TokenType::SymbolKey, &self.input[start..self.position]);
                    }

                    // Return the identifier which may also be a reserved keyword
                    return Token::new(lookup_identifier(&literal), literal);
                } else if Self::is_digit(self.ch) {
                    let (token_type, literal) = self.read_number();

                    return Token::new(token_type, literal);
                } else {
                    Token::new(
                        TokenType::Illegal,
                        &self.input[self.position..self.position + 1],
                    )
                }
            }
        };

        self.read_ascii_char();
        token
    }

    fn skip_whitespace(&mut self) {
        while WHITESPACE_CHARS.contains(&self.ch) {
            self.read_ascii_char();
        }
    }

    pub fn read_ascii_char(&mut self) {
        if self.read_position >= self.input.len() {
            self.ch = '\0';
        } else {
            self.ch = self.input.as_bytes()[self.read_position] as char;
        }

        self.position = self.read_position;
        self.read_position += 1;
    }

    pub fn read_unicode_char(&mut self) {
        if self.read_position >= self.input.len() {
            self.ch = '\0';
            self.position = self.read_position;

            return;
        } else {
            let slice = &self.input[self.read_position..];

            if let Some(ch) = slice.chars().next() {
                self.ch = ch;
                self.position = self.read_position;
                self.read_position += ch.len_utf8();
            } else {
                self.ch = '\0';
                self.position = self.read_position;
            }
        }
    }

    pub fn peek_ascii_char(&self) -> char {
        if self.read_position >= self.input.len() {
            '\0'
        } else {
            self.input.as_bytes()[self.read_position] as char
        }
    }

    pub fn peek_unicode_char(&self) -> char {
        if self.read_position >= self.input.len() {
            return '\0';
        } else {
            let slice = &self.input[self.read_position..];

            if let Some(ch) = slice.chars().next() {
                return ch;
            } else {
                return '\0';
            }
        }
    }

    fn read_operator(&mut self) -> &'a str {
        let start = self.position;

        // Advance to the next character to complete the operator
        self.read_ascii_char();

        &self.input[start..self.position + 1]
    }

    fn read_comment(&mut self) -> &'a str {
        let start = self.position;

        // Get everything from current position to end
        let remaining = &self.input[self.position..];
        let mut chars = remaining.chars();

        // Consume characters until end of line or end of input
        while let Some(ch) = chars.next() {
            if ch == '\n' {
                break;
            }

            self.read_unicode_char();
        }

        // Return slice from start to current position
        &self.input[start..self.position]
    }

    fn read_string(&mut self) -> (TokenType, &'a str) {
        let position = self.position; // Start position includes the opening quote

        loop {
            self.read_unicode_char();

            // Handle escaped quotes and do not terminate the string prematurely
            if self.ch == '\\' && self.peek_unicode_char() == '"' {
                self.read_unicode_char(); // Skip the escaped quote
                continue;
            }

            // String ends with closing quote or unterminated at EOF
            if self.ch == '"' || self.ch == '\0' {
                break;
            }
        }

        // Move past closing quote if one is found
        if self.ch == '"' {
            self.read_unicode_char();
        }

        let literal = &self.input[position..self.position];

        // Check if the string was properly terminated (has at least opening and closing quotes)
        if literal.len() >= 2 && literal.ends_with('"') {
            (TokenType::String, literal)
        } else {
            (TokenType::Illegal, literal)
        }
    }

    fn read_symbol(&mut self) -> (TokenType, &'a str) {
        let start = self.position;
        self.read_unicode_char(); // Start position after the ':'

        if self.ch.is_alphabetic() || self.ch == '_' {
            // Read the symbol like an identifier
            self.read_identifier();

            return (TokenType::Symbol, &self.input[start..self.position]);
        } else if Self::is_digit(self.ch)
            || (!self.ch.is_alphabetic() && !Self::is_whitespace(self.ch))
        // If the symbol starts with a number, or some special character
        {
            // Read until we hit an alphabetic character or whitespace
            while Self::is_digit(self.ch) || !self.ch.is_alphabetic() {
                self.read_unicode_char();
            }

            return (TokenType::Illegal, &self.input[start..self.position]);
        } else {
            return (TokenType::Illegal, &self.input[start..self.position]);
        }
    }

    fn read_identifier(&mut self) -> &'a str {
        let position = self.position;

        while self.ch.is_alphanumeric() || self.ch == '_' {
            self.read_unicode_char();
        }

        if SUFFIX_CHARS.contains(&self.ch) {
            self.read_unicode_char();
        }

        &self.input[position..self.position]
    }

    fn is_digit(ch: char) -> bool {
        ch.is_digit(10)
    }

    fn is_whitespace(ch: char) -> bool {
        WHITESPACE_CHARS.contains(&ch)
    }

    fn read_number(&mut self) -> (TokenType, &'a str) {
        let position = self.position;

        // Read integer part and check for errors
        let integer_part_valid = self.read_number_part();

        // Check if we have a decimal point followed by a digit
        if self.ch == '.' && Self::is_digit(self.peek_ascii_char()) {
            self.read_ascii_char(); // consume '.'

            let decimal_part_valid = self.read_number_part();

            let literal = &self.input[position..self.position];

            if integer_part_valid && decimal_part_valid {
                return (TokenType::Float, literal);
            } else {
                return (TokenType::Illegal, literal);
            }
        }

        let literal = &self.input[position..self.position];

        if integer_part_valid {
            (TokenType::Integer, literal)
        } else {
            (TokenType::Illegal, literal)
        }
    }

    // Helper method to read a sequence of digits with optional underscores
    // Returns false if the sequence is invalid (consecutive underscores or trailing underscore)
    fn read_number_part(&mut self) -> bool {
        let mut last_was_underscore = false;
        let mut has_digits = false;

        while Self::is_digit(self.ch) || self.ch == '_' {
            if self.ch == '_' {
                if last_was_underscore {
                    // Consecutive underscores - consume rest of number and mark as invalid
                    while Self::is_digit(self.ch) || self.ch == '_' {
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
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Token<'a>> {
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
    use super::Lexer;
    use crate::token::TokenType;

    fn assert_tokens(expected: Vec<(TokenType, &str)>, input: &str) {
        let tokens: Vec<_> = Lexer::new(input)
            .map(|t| (t.token_type, t.literal))
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
    fn unterminated_string() {
        let input = r#""This is an unterminated string"#;

        let expected = vec![(TokenType::Illegal, "\"This is an unterminated string")];

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
