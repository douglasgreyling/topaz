use crate::token::{Token, TokenType, lookup_identifier};

const WHITESPACE_CHARS: [char; 4] = [' ', '\t', '\n', '\r'];
const SUFFIX_CHARS: [char; 2] = ['!', '?'];

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
                        self.read_char(); // Advance past the colon
                        return Token::new(TokenType::SymbolKey, format!("{}:", literal));
                    }

                    // Return the identifier which may also be a reserved keyword
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
        while WHITESPACE_CHARS.contains(&self.ch) {
            self.read_char();
        }
    }

    fn read_char(&mut self) {
        // Return a null character if we've reached the end of the input
        if self.input.len() <= self.read_position {
            self.ch = '\0';
        } else {
            self.ch = self.input[self.read_position];
        }

        self.position = self.read_position;
        self.read_position += 1;
    }

    fn peek_char(&self) -> char {
        // Return a null character if we've reached the end of the input
        if self.input.len() <= self.read_position {
            '\0'
        } else {
            self.input[self.read_position]
        }
    }

    fn read_operator(&mut self) -> String {
        let first_char = self.ch; // Start with the current character

        // Advance to the next character to complete the operator
        self.read_char();

        format!("{}{}", first_char, self.ch)
    }

    fn read_comment(&mut self) -> String {
        let position = self.position;

        self.read_char(); // Move past the '#' character

        // Read until end of line or end of file
        while self.ch != '\n' && self.ch != '\0' {
            self.read_char();
        }

        self.input[position..self.position].iter().collect()
    }

    fn read_string(&mut self) -> (TokenType, String) {
        let position = self.position; // Start position includes the opening quote

        loop {
            self.read_char();

            // Handle escaped quotes and do not terminate the string prematurely
            if self.ch == '\\' && self.peek_char() == '"' {
                self.read_char(); // Skip the escaped quote
                continue;
            }

            // String ends with closing quote or unterminated at EOF
            if self.ch == '"' || self.ch == '\0' {
                break;
            }
        }

        // Move past closing quote if one is found
        if self.ch == '"' {
            self.read_char();
        }

        let literal: String = self.input[position..self.position].iter().collect();

        // Check if the string was properly terminated (has at least opening and closing quotes)
        if literal.len() >= 2 && literal.ends_with('"') {
            (TokenType::String, literal)
        } else {
            (
                TokenType::Illegal,
                format!("unterminated string: {}", literal),
            )
        }
    }

    fn read_symbol(&mut self) -> (TokenType, String) {
        self.read_char(); // Start position after the ':'

        if self.ch.is_alphabetic() || self.ch == '_' {
            // Read the symbol like an identifier
            let literal = self.read_identifier();

            return (TokenType::Symbol, format!(":{}", literal));
        } else if Self::is_digit(self.ch)
            || (!self.ch.is_alphabetic() && !Self::is_whitespace(self.ch))
        // If the symbol starts with a number, or some special character
        {
            let position = self.position;

            // Read until we hit an alphabetic character or whitespace
            while Self::is_digit(self.ch) || !self.ch.is_alphabetic() {
                self.read_char();
            }

            let literal: String = self.input[position..self.position].iter().collect();

            return (TokenType::Illegal, format!(":{}", literal));
        } else {
            return (TokenType::Illegal, ":".to_string());
        }
    }

    fn read_identifier(&mut self) -> String {
        let position = self.position;

        while self.ch.is_alphanumeric() || self.ch == '_' {
            self.read_char();
        }

        if SUFFIX_CHARS.contains(&self.ch) {
            self.read_char();
        }

        self.input[position..self.position].iter().collect()
    }

    fn is_digit(ch: char) -> bool {
        ch.is_digit(10)
    }

    fn is_whitespace(ch: char) -> bool {
        WHITESPACE_CHARS.contains(&ch)
    }

    fn read_number(&mut self) -> (TokenType, String) {
        let position = self.position;
        let mut last_was_underscore = false;

        // Read integer part and check for errors
        let integer_part_valid = self.read_number_part();

        // Check if we have a decimal point followed by a digit
        if self.ch == '.' && Self::is_digit(self.peek_char()) {
            self.read_char(); // consume '.'

            let decimal_part_valid = self.read_number_part();

            let literal = self.input[position..self.position].iter().collect();

            if integer_part_valid && decimal_part_valid {
                return (TokenType::Float, literal);
            } else {
                return (TokenType::Illegal, literal);
            }
        }

        let literal = self.input[position..self.position].iter().collect();

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
                        self.read_char();
                    }

                    return false;
                }

                last_was_underscore = true;
            } else {
                last_was_underscore = false;
                has_digits = true;
            }

            self.read_char();
        }

        // Number part ending with underscore is invalid
        !last_was_underscore && has_digits
    }
}

impl Iterator for Lexer {
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
    use super::Lexer;
    use crate::token::TokenType;

    fn assert_tokens(expected: Vec<(TokenType, String)>, input: &str) {
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
            (TokenType::Plus, "+".to_string()),
            (TokenType::Minus, "-".to_string()),
            (TokenType::Asterisk, "*".to_string()),
            (TokenType::Slash, "/".to_string()),
            (TokenType::Power, "**".to_string()),
            (TokenType::HashRocket, "=>".to_string()),
            (TokenType::LTE, "<=".to_string()),
            (TokenType::Eq, "==".to_string()),
            (TokenType::GTE, ">=".to_string()),
            (TokenType::NotEq, "!=".to_string()),
            (TokenType::LT, "<".to_string()),
            (TokenType::GT, ">".to_string()),
            (TokenType::And, "&&".to_string()),
            (TokenType::Or, "||".to_string()),
            (TokenType::Bang, "!".to_string()),
        ];

        assert_tokens(expected, input);
    }

    #[test]
    fn unary_operators() {
        let input = "!true -4";

        let expected = vec![
            (TokenType::Bang, "!".to_string()),
            (TokenType::True, "true".to_string()),
            (TokenType::Minus, "-".to_string()),
            (TokenType::Integer, "4".to_string()),
        ];

        assert_tokens(expected, input);
    }

    #[test]
    fn plus_variations() {
        let input = "+ ++ +a a+";

        let expected = vec![
            (TokenType::Plus, "+".to_string()),
            (TokenType::Plus, "+".to_string()),
            (TokenType::Plus, "+".to_string()),
            (TokenType::Plus, "+".to_string()),
            (TokenType::Identifier, "a".to_string()),
            (TokenType::Identifier, "a".to_string()),
            (TokenType::Plus, "+".to_string()),
        ];

        assert_tokens(expected, input);
    }

    #[test]
    fn minus_variations() {
        let input = "- -- -a a-";

        let expected = vec![
            (TokenType::Minus, "-".to_string()),
            (TokenType::Minus, "-".to_string()),
            (TokenType::Minus, "-".to_string()),
            (TokenType::Minus, "-".to_string()),
            (TokenType::Identifier, "a".to_string()),
            (TokenType::Identifier, "a".to_string()),
            (TokenType::Minus, "-".to_string()),
        ];

        assert_tokens(expected, input);
    }

    #[test]
    fn asterisk_variations() {
        let input = "* ** *a a*";

        let expected = vec![
            (TokenType::Asterisk, "*".to_string()),
            (TokenType::Power, "**".to_string()),
            (TokenType::Asterisk, "*".to_string()),
            (TokenType::Identifier, "a".to_string()),
            (TokenType::Identifier, "a".to_string()),
            (TokenType::Asterisk, "*".to_string()),
        ];

        assert_tokens(expected, input);
    }

    #[test]
    fn slash_variations() {
        let input = "/ // /a a/";

        let expected = vec![
            (TokenType::Slash, "/".to_string()),
            (TokenType::Slash, "/".to_string()),
            (TokenType::Slash, "/".to_string()),
            (TokenType::Slash, "/".to_string()),
            (TokenType::Identifier, "a".to_string()),
            (TokenType::Identifier, "a".to_string()),
            (TokenType::Slash, "/".to_string()),
        ];

        assert_tokens(expected, input);
    }

    #[test]
    fn power_variations() {
        let input = "** *** **a a**";

        let expected = vec![
            (TokenType::Power, "**".to_string()),
            (TokenType::Power, "**".to_string()),
            (TokenType::Asterisk, "*".to_string()),
            (TokenType::Power, "**".to_string()),
            (TokenType::Identifier, "a".to_string()),
            (TokenType::Identifier, "a".to_string()),
            (TokenType::Power, "**".to_string()),
        ];

        assert_tokens(expected, input);
    }

    #[test]
    fn hash_rocket_variations() {
        let input = "=> ==> =>= =>> =>< <=> =>a a=>";

        let expected = vec![
            (TokenType::HashRocket, "=>".to_string()),
            (TokenType::Eq, "==".to_string()),
            (TokenType::GT, ">".to_string()),
            (TokenType::HashRocket, "=>".to_string()),
            (TokenType::Assign, "=".to_string()),
            (TokenType::HashRocket, "=>".to_string()),
            (TokenType::GT, ">".to_string()),
            (TokenType::HashRocket, "=>".to_string()),
            (TokenType::LT, "<".to_string()),
            (TokenType::LTE, "<=".to_string()),
            (TokenType::GT, ">".to_string()),
            (TokenType::HashRocket, "=>".to_string()),
            (TokenType::Identifier, "a".to_string()),
            (TokenType::Identifier, "a".to_string()),
            (TokenType::HashRocket, "=>".to_string()),
        ];

        assert_tokens(expected, input);
    }

    #[test]
    fn lte_variations() {
        let input = "<= <<= <== =<= <<= ><= <=a a<=";

        let expected = vec![
            (TokenType::LTE, "<=".to_string()),
            (TokenType::LT, "<".to_string()),
            (TokenType::LTE, "<=".to_string()),
            (TokenType::LTE, "<=".to_string()),
            (TokenType::Assign, "=".to_string()),
            (TokenType::Assign, "=".to_string()),
            (TokenType::LTE, "<=".to_string()),
            (TokenType::LT, "<".to_string()),
            (TokenType::LTE, "<=".to_string()),
            (TokenType::GT, ">".to_string()),
            (TokenType::LTE, "<=".to_string()),
            (TokenType::LTE, "<=".to_string()),
            (TokenType::Identifier, "a".to_string()),
            (TokenType::Identifier, "a".to_string()),
            (TokenType::LTE, "<=".to_string()),
        ];

        assert_tokens(expected, input);
    }

    #[test]
    fn equality_variations() {
        let input = "= == === =a ==a a==";

        let expected = vec![
            (TokenType::Assign, "=".to_string()),
            (TokenType::Eq, "==".to_string()),
            (TokenType::Eq, "==".to_string()),
            (TokenType::Assign, "=".to_string()),
            (TokenType::Assign, "=".to_string()),
            (TokenType::Identifier, "a".to_string()),
            (TokenType::Eq, "==".to_string()),
            (TokenType::Identifier, "a".to_string()),
            (TokenType::Identifier, "a".to_string()),
            (TokenType::Eq, "==".to_string()),
        ];

        assert_tokens(expected, input);
    }

    #[test]
    fn gte_variations() {
        let input = ">= >== =>= >=< >>= >=a a>=";

        let expected = vec![
            (TokenType::GTE, ">=".to_string()),
            (TokenType::GTE, ">=".to_string()),
            (TokenType::Assign, "=".to_string()),
            (TokenType::HashRocket, "=>".to_string()),
            (TokenType::Assign, "=".to_string()),
            (TokenType::GTE, ">=".to_string()),
            (TokenType::LT, "<".to_string()),
            (TokenType::GT, ">".to_string()),
            (TokenType::GTE, ">=".to_string()),
            (TokenType::GTE, ">=".to_string()),
            (TokenType::Identifier, "a".to_string()),
            (TokenType::Identifier, "a".to_string()),
            (TokenType::GTE, ">=".to_string()),
        ];

        assert_tokens(expected, input);
    }

    #[test]
    fn lt_and_gt_variations() {
        let input = "< << >< <> > >> <a a> <a a<";

        let expected = vec![
            (TokenType::LT, "<".to_string()),
            (TokenType::LT, "<".to_string()),
            (TokenType::LT, "<".to_string()),
            (TokenType::GT, ">".to_string()),
            (TokenType::LT, "<".to_string()),
            (TokenType::LT, "<".to_string()),
            (TokenType::GT, ">".to_string()),
            (TokenType::GT, ">".to_string()),
            (TokenType::GT, ">".to_string()),
            (TokenType::GT, ">".to_string()),
            (TokenType::LT, "<".to_string()),
            (TokenType::Identifier, "a".to_string()),
            (TokenType::Identifier, "a".to_string()),
            (TokenType::GT, ">".to_string()),
            (TokenType::LT, "<".to_string()),
            (TokenType::Identifier, "a".to_string()),
            (TokenType::Identifier, "a".to_string()),
            (TokenType::LT, "<".to_string()),
        ];

        assert_tokens(expected, input);
    }

    #[test]
    fn and_variations() {
        let input = "&& &&& &&&& &&a a&&";

        let expected = vec![
            (TokenType::And, "&&".to_string()),
            (TokenType::And, "&&".to_string()),
            (TokenType::Illegal, "&".to_string()),
            (TokenType::And, "&&".to_string()),
            (TokenType::And, "&&".to_string()),
            (TokenType::And, "&&".to_string()),
            (TokenType::Identifier, "a".to_string()),
            (TokenType::Identifier, "a".to_string()),
            (TokenType::And, "&&".to_string()),
        ];

        assert_tokens(expected, input);
    }

    #[test]
    fn or_variations() {
        let input = "|| ||| |||| ||a a||";

        let expected = vec![
            (TokenType::Or, "||".to_string()),
            (TokenType::Or, "||".to_string()),
            (TokenType::Illegal, "|".to_string()),
            (TokenType::Or, "||".to_string()),
            (TokenType::Or, "||".to_string()),
            (TokenType::Or, "||".to_string()),
            (TokenType::Identifier, "a".to_string()),
            (TokenType::Identifier, "a".to_string()),
            (TokenType::Or, "||".to_string()),
        ];

        assert_tokens(expected, input);
    }

    #[test]
    fn not_variations() {
        let input = "! != !! !a a!";

        let expected = vec![
            (TokenType::Bang, "!".to_string()),
            (TokenType::NotEq, "!=".to_string()),
            (TokenType::Bang, "!".to_string()),
            (TokenType::Bang, "!".to_string()),
            (TokenType::Bang, "!".to_string()),
            (TokenType::Identifier, "a".to_string()),
            (TokenType::Identifier, "a!".to_string()),
        ];

        assert_tokens(expected, input);
    }

    #[test]
    fn special_characters() {
        let input = "; { } [ ] , . | a; ;a a{ {a a} }a a[ [a a] ]a a, ,a a. .a";

        let expected = vec![
            (TokenType::Semicolon, ";".to_string()),
            (TokenType::LBrace, "{".to_string()),
            (TokenType::RBrace, "}".to_string()),
            (TokenType::LBrack, "[".to_string()),
            (TokenType::RBrack, "]".to_string()),
            (TokenType::Comma, ",".to_string()),
            (TokenType::Dot, ".".to_string()),
            (TokenType::Illegal, "|".to_string()),
            (TokenType::Identifier, "a".to_string()),
            (TokenType::Semicolon, ";".to_string()),
            (TokenType::Semicolon, ";".to_string()),
            (TokenType::Identifier, "a".to_string()),
            (TokenType::Identifier, "a".to_string()),
            (TokenType::LBrace, "{".to_string()),
            (TokenType::LBrace, "{".to_string()),
            (TokenType::Identifier, "a".to_string()),
            (TokenType::Identifier, "a".to_string()),
            (TokenType::RBrace, "}".to_string()),
            (TokenType::RBrace, "}".to_string()),
            (TokenType::Identifier, "a".to_string()),
            (TokenType::Identifier, "a".to_string()),
            (TokenType::LBrack, "[".to_string()),
            (TokenType::LBrack, "[".to_string()),
            (TokenType::Identifier, "a".to_string()),
            (TokenType::Identifier, "a".to_string()),
            (TokenType::RBrack, "]".to_string()),
            (TokenType::RBrack, "]".to_string()),
            (TokenType::Identifier, "a".to_string()),
            (TokenType::Identifier, "a".to_string()),
            (TokenType::Comma, ",".to_string()),
            (TokenType::Comma, ",".to_string()),
            (TokenType::Identifier, "a".to_string()),
            (TokenType::Identifier, "a".to_string()),
            (TokenType::Dot, ".".to_string()),
            (TokenType::Dot, ".".to_string()),
            (TokenType::Identifier, "a".to_string()),
        ];

        assert_tokens(expected, input);
    }

    #[test]
    fn keywords() {
        let input = "def do end if else elsif while return true false nil DEF DO END IF ELSE ELSIF WHILE RETURN TRUE FALSE NIL";

        let expected = vec![
            (TokenType::Def, "def".to_string()),
            (TokenType::Do, "do".to_string()),
            (TokenType::End, "end".to_string()),
            (TokenType::If, "if".to_string()),
            (TokenType::Else, "else".to_string()),
            (TokenType::Elsif, "elsif".to_string()),
            (TokenType::While, "while".to_string()),
            (TokenType::Return, "return".to_string()),
            (TokenType::True, "true".to_string()),
            (TokenType::False, "false".to_string()),
            (TokenType::Nil, "nil".to_string()),
            (TokenType::Identifier, "DEF".to_string()),
            (TokenType::Identifier, "DO".to_string()),
            (TokenType::Identifier, "END".to_string()),
            (TokenType::Identifier, "IF".to_string()),
            (TokenType::Identifier, "ELSE".to_string()),
            (TokenType::Identifier, "ELSIF".to_string()),
            (TokenType::Identifier, "WHILE".to_string()),
            (TokenType::Identifier, "RETURN".to_string()),
            (TokenType::Identifier, "TRUE".to_string()),
            (TokenType::Identifier, "FALSE".to_string()),
            (TokenType::Identifier, "NIL".to_string()),
        ];

        assert_tokens(expected, input);
    }

    #[test]
    fn keywords_with_suffixes() {
        let input = "defx endian donut iffy elsify truex falsenil nilly";

        let expected = vec![
            (TokenType::Identifier, "defx".to_string()),
            (TokenType::Identifier, "endian".to_string()),
            (TokenType::Identifier, "donut".to_string()),
            (TokenType::Identifier, "iffy".to_string()),
            (TokenType::Identifier, "elsify".to_string()),
            (TokenType::Identifier, "truex".to_string()),
            (TokenType::Identifier, "falsenil".to_string()),
            (TokenType::Identifier, "nilly".to_string()),
        ];

        assert_tokens(expected, input);
    }

    #[test]
    fn identifiers() {
        let c = "?".chars().next().unwrap();
        println!("{}", c.is_alphanumeric());

        let input = "_ _test test_ __private__ test! test1 1test test? !test test_1 test_! test_? 1_test !_test foo.bar.baz Hello_世界 café";

        let expected = vec![
            (TokenType::Identifier, "_".to_string()),
            (TokenType::Identifier, "_test".to_string()),
            (TokenType::Identifier, "test_".to_string()),
            (TokenType::Identifier, "__private__".to_string()),
            (TokenType::Identifier, "test!".to_string()),
            (TokenType::Identifier, "test1".to_string()),
            (TokenType::Integer, "1".to_string()),
            (TokenType::Identifier, "test".to_string()),
            (TokenType::Identifier, "test?".to_string()),
            (TokenType::Bang, "!".to_string()),
            (TokenType::Identifier, "test".to_string()),
            (TokenType::Identifier, "test_1".to_string()),
            (TokenType::Identifier, "test_!".to_string()),
            (TokenType::Identifier, "test_?".to_string()),
            (TokenType::Illegal, "1_".to_string()),
            (TokenType::Identifier, "test".to_string()),
            (TokenType::Bang, "!".to_string()),
            (TokenType::Identifier, "_test".to_string()),
            (TokenType::Identifier, "foo".to_string()),
            (TokenType::Dot, ".".to_string()),
            (TokenType::Identifier, "bar".to_string()),
            (TokenType::Dot, ".".to_string()),
            (TokenType::Identifier, "baz".to_string()),
            (TokenType::Identifier, "Hello_世界".to_string()),
            (TokenType::Identifier, "café".to_string()),
        ];

        assert_tokens(expected, input);
    }

    #[test]
    fn symbols() {
        let input = ": :_ :_test :test_ :__private__ :test! :test1 :1test :test? :!test :test_1 :test_! :test_? :1_test :!_test :!!_test";

        let expected = vec![
            (TokenType::Illegal, ":".to_string()),
            (TokenType::Symbol, ":_".to_string()),
            (TokenType::Symbol, ":_test".to_string()),
            (TokenType::Symbol, ":test_".to_string()),
            (TokenType::Symbol, ":__private__".to_string()),
            (TokenType::Symbol, ":test!".to_string()),
            (TokenType::Symbol, ":test1".to_string()),
            (TokenType::Illegal, ":1".to_string()),
            (TokenType::Identifier, "test".to_string()),
            (TokenType::Symbol, ":test?".to_string()),
            (TokenType::Illegal, ":!".to_string()),
            (TokenType::Identifier, "test".to_string()),
            (TokenType::Symbol, ":test_1".to_string()),
            (TokenType::Symbol, ":test_!".to_string()),
            (TokenType::Symbol, ":test_?".to_string()),
            (TokenType::Illegal, ":1_".to_string()),
            (TokenType::Identifier, "test".to_string()),
            (TokenType::Illegal, ":!_".to_string()),
            (TokenType::Identifier, "test".to_string()),
            (TokenType::Illegal, ":!!_".to_string()),
            (TokenType::Identifier, "test".to_string()),
        ];

        assert_tokens(expected, input);
    }

    #[test]
    fn literals() {
        let input =
            "0 42 3.14 0.0 -5 -5.12 01 00 001.2 1_2_3_4 1_2_3_4.1 1.1_2_3 1_ 1._2 1_.2 1.2.3";

        let expected = vec![
            (TokenType::Integer, "0".to_string()),
            (TokenType::Integer, "42".to_string()),
            (TokenType::Float, "3.14".to_string()),
            (TokenType::Float, "0.0".to_string()),
            (TokenType::Minus, "-".to_string()),
            (TokenType::Integer, "5".to_string()),
            (TokenType::Minus, "-".to_string()),
            (TokenType::Float, "5.12".to_string()),
            (TokenType::Integer, "01".to_string()),
            (TokenType::Integer, "00".to_string()),
            (TokenType::Float, "001.2".to_string()),
            (TokenType::Integer, "1_2_3_4".to_string()),
            (TokenType::Float, "1_2_3_4.1".to_string()),
            (TokenType::Float, "1.1_2_3".to_string()),
            (TokenType::Illegal, "1_".to_string()),
            (TokenType::Integer, "1".to_string()),
            (TokenType::Dot, ".".to_string()),
            (TokenType::Illegal, "_2".to_string()),
            (TokenType::Illegal, "1_.2".to_string()),
            (TokenType::Float, "1.2".to_string()),
            (TokenType::Dot, ".".to_string()),
            (TokenType::Integer, "3".to_string()),
        ];

        assert_tokens(expected, input);
    }

    #[test]
    fn strings() {
        let input = r#""hello" "hello world" "h\"ello" "5.7" "Hello 世界" "café" ""#;

        let expected = vec![
            (TokenType::String, "\"hello\"".to_string()),
            (TokenType::String, "\"hello world\"".to_string()),
            (TokenType::String, "\"h\\\"ello\"".to_string()),
            (TokenType::String, "\"5.7\"".to_string()),
            (TokenType::String, "\"Hello 世界\"".to_string()),
            (TokenType::String, "\"café\"".to_string()),
            (TokenType::Illegal, "unterminated string: \"".to_string()),
        ];

        assert_tokens(expected, input);
    }

    #[test]
    fn unterminated_string() {
        let input = r#""This is an unterminated string"#;

        let expected = vec![(
            TokenType::Illegal,
            "unterminated string: \"This is an unterminated string".to_string(),
        )];

        assert_tokens(expected, input);
    }

    #[test]
    fn single_quote() {
        let input = r#"""#;

        let expected = vec![(TokenType::Illegal, "unterminated string: \"".to_string())];

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
            (TokenType::Comment, "# This is a comment".to_string()),
            (
                TokenType::Comment,
                "# This is a comment with a # inside".to_string(),
            ),
            (TokenType::Comment, "#Another comment".to_string()),
            (TokenType::Comment, "#    Indented comment".to_string()),
            (TokenType::Comment, "#".to_string()),
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
            (TokenType::Identifier, "x".to_string()),
            (TokenType::Assign, "=".to_string()),
            (TokenType::Integer, "5".to_string()),
            (TokenType::Plus, "+".to_string()),
            (TokenType::Integer, "3".to_string()),
            (TokenType::Identifier, "x".to_string()),
            (TokenType::Assign, "=".to_string()),
            (TokenType::Integer, "5".to_string()),
            (TokenType::Identifier, "x".to_string()),
            (TokenType::Assign, "=".to_string()),
            (TokenType::Integer, "5".to_string()),
            (TokenType::Identifier, "x".to_string()),
            (TokenType::Assign, "=".to_string()),
            (TokenType::Integer, "5".to_string()),
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
            (TokenType::Comment, "# Types".to_string()),
            (TokenType::Integer, "1".to_string()),
            (TokenType::Float, "4.0".to_string()),
            (TokenType::Symbol, ":my_symbol".to_string()),
            (TokenType::String, "\"This is a string\"".to_string()),
            (TokenType::Comment, "# Binary Operators".to_string()),
            (TokenType::Integer, "1".to_string()),
            (TokenType::Plus, "+".to_string()),
            (TokenType::Integer, "2".to_string()),
            (TokenType::Asterisk, "*".to_string()),
            (TokenType::LParen, "(".to_string()),
            (TokenType::Integer, "3".to_string()),
            (TokenType::Minus, "-".to_string()),
            (TokenType::Integer, "4".to_string()),
            (TokenType::RParen, ")".to_string()),
            (TokenType::Slash, "/".to_string()),
            (TokenType::Integer, "5".to_string()),
            (TokenType::Integer, "2".to_string()),
            (TokenType::Power, "**".to_string()),
            (TokenType::Integer, "3".to_string()),
            (TokenType::Identifier, "x".to_string()),
            (TokenType::Eq, "==".to_string()),
            (TokenType::Identifier, "y".to_string()),
            (TokenType::Identifier, "x".to_string()),
            (TokenType::NotEq, "!=".to_string()),
            (TokenType::Identifier, "y".to_string()),
            (TokenType::Identifier, "x".to_string()),
            (TokenType::GTE, ">=".to_string()),
            (TokenType::Integer, "10".to_string()),
            (TokenType::Identifier, "x".to_string()),
            (TokenType::LTE, "<=".to_string()),
            (TokenType::Integer, "10".to_string()),
            (TokenType::Identifier, "x".to_string()),
            (TokenType::LT, "<".to_string()),
            (TokenType::Integer, "5".to_string()),
            (TokenType::Identifier, "x".to_string()),
            (TokenType::GT, ">".to_string()),
            (TokenType::Integer, "2".to_string()),
            (TokenType::Comment, "# Functions".to_string()),
            (TokenType::Def, "def".to_string()),
            (TokenType::Identifier, "hello_world".to_string()),
            (TokenType::LParen, "(".to_string()),
            (TokenType::Identifier, "name".to_string()),
            (TokenType::RParen, ")".to_string()),
            (TokenType::Do, "do".to_string()),
            (TokenType::Return, "return".to_string()),
            (TokenType::Identifier, "puts".to_string()),
            (TokenType::String, "\"Hello, #{name}!\"".to_string()),
            (TokenType::End, "end".to_string()),
            (TokenType::Identifier, "object".to_string()),
            (TokenType::Dot, ".".to_string()),
            (TokenType::Identifier, "method_name".to_string()),
            (TokenType::LParen, "(".to_string()),
            (TokenType::Integer, "1".to_string()),
            (TokenType::RParen, ")".to_string()),
            (TokenType::Comment, "# Control Flow".to_string()),
            (TokenType::If, "if".to_string()),
            (TokenType::Identifier, "x".to_string()),
            (TokenType::GT, ">".to_string()),
            (TokenType::Integer, "5".to_string()),
            (TokenType::Do, "do".to_string()),
            (TokenType::Identifier, "result".to_string()),
            (TokenType::Assign, "=".to_string()),
            (TokenType::Identifier, "x".to_string()),
            (TokenType::Plus, "+".to_string()),
            (TokenType::Identifier, "y".to_string()),
            (TokenType::Elsif, "elsif".to_string()),
            (TokenType::Identifier, "y".to_string()),
            (TokenType::LT, "<".to_string()),
            (TokenType::Integer, "10".to_string()),
            (TokenType::Do, "do".to_string()),
            (TokenType::Identifier, "result".to_string()),
            (TokenType::Assign, "=".to_string()),
            (TokenType::Identifier, "x".to_string()),
            (TokenType::Minus, "-".to_string()),
            (TokenType::Identifier, "y".to_string()),
            (TokenType::Else, "else".to_string()),
            (TokenType::Do, "do".to_string()),
            (TokenType::Nil, "nil".to_string()),
            (TokenType::End, "end".to_string()),
            (TokenType::While, "while".to_string()),
            (TokenType::Identifier, "x".to_string()),
            (TokenType::LT, "<".to_string()),
            (TokenType::Integer, "10".to_string()),
            (TokenType::Do, "do".to_string()),
            (TokenType::Identifier, "x".to_string()),
            (TokenType::Assign, "=".to_string()),
            (TokenType::Identifier, "x".to_string()),
            (TokenType::Plus, "+".to_string()),
            (TokenType::Integer, "1".to_string()),
            (TokenType::End, "end".to_string()),
            (TokenType::Comment, "# Boolean logic".to_string()),
            (TokenType::True, "true".to_string()),
            (TokenType::And, "&&".to_string()),
            (TokenType::False, "false".to_string()),
            (TokenType::True, "true".to_string()),
            (TokenType::Or, "||".to_string()),
            (TokenType::False, "false".to_string()),
            (TokenType::Bang, "!".to_string()),
            (TokenType::True, "true".to_string()),
            (TokenType::Comment, "# Hashes".to_string()),
            (TokenType::LBrace, "{".to_string()),
            (TokenType::SymbolKey, "name:".to_string()),
            (TokenType::String, "\"John\"".to_string()),
            (TokenType::Comma, ",".to_string()),
            (TokenType::Identifier, "age".to_string()),
            (TokenType::HashRocket, "=>".to_string()),
            (TokenType::Integer, "30".to_string()),
            (TokenType::Comma, ",".to_string()),
            (TokenType::String, "\"height\"".to_string()),
            (TokenType::HashRocket, "=>".to_string()),
            (TokenType::Float, "5.9".to_string()),
            (TokenType::RBrace, "}".to_string()),
        ];

        assert_tokens(expected, input);
    }
}
