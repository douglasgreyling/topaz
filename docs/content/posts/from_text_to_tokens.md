+++
date = '2026-01-16T12:34:03+02:00'
draft = false
title = 'From text to tokens'
tags = ['lexing', 'rust', 'token']
+++

Hey there! 👋

Let's talk about lexers, and more specifically, how the Topaz lexer works. If you've ever wondered how programming languages make sense of the code you write you'll catch a little bit of a glimpse of it in this post.

## What's a Lexer, Anyway?

Think back to when you first learned to read. Before you could read words, you had to learn to recognize letters and sounds. Once you learned to read words, you could understand sentences and were then able to read and write meaning into them. A lexer is the first step in that process.

A lexer is like someone who reads through your code with a highlighter, marking different parts and saying "this is a number," "this is a word," "this is an equals sign," and so on. It breaks down your code into little chunks called **tokens**, each labeled with what it represents.

For example, if you write:

```ruby
x = 42 + 10
```

The lexer sees:
- `x` → an identifier (a name for something)
- `=` → an assignment operator
- `42` → an integer
- `+` → a plus operator
- `10` → another integer

This process transforms raw text into a structured list of tokens that can later be parsed and understood by the language interpreter.

## The Topaz Lexer Architecture

The Topaz lexer is built in Rust and at its heart is a struct that keeps track of four key things:

```rust
pub struct Lexer {
    input: Vec<char>,      // The code as a list of characters
    position: usize,       // Where we are right now
    read_position: usize,  // Where we're looking ahead to
    ch: char,              // The current character we're examining
}
```

The lexer reads the input code one character at a time, using `position` to track where it currently is and `read_position` to peek ahead. The `ch` field holds the current character being analyzed.

This two-position system (current position + read ahead position) is helpful because it lets the lexer peek at the next character without committing to moving forward onto the next character in the iteration. This is crucial for recognizing tokens which are made up of multiple characters.

## How It All Works: A Character-by-Character Journey

When you instantiate a new lexer with some code, it immediately reads the first character and gets ready to work. The main workhorse is the `next_token()` method, which is called repeatedly to produce one token after another.

Here's the basic flow:

### Step 1: Skip the Boring Stuff

First, the lexer skips over whitespace (spaces, tabs, newlines). These are important for humans but don't affect the meaning of the code:

```rust
fn skip_whitespace(&mut self) {
    while WHITESPACE_CHARS.contains(&self.ch) {
        self.read_char();
    }
}
```

### Step 2: Pattern Match on the Current Character

This is where the magic happens! The lexer looks at the current character and decides what to do with a big match expression. Let's walk through some interesting cases:

#### Simple Single-Character Tokens

Some characters always mean the same thing:
- `(` is always a left parenthesis
- `)` is always a right parenthesis
- `+` is always a plus sign
- `,` is always a comma

These are straightforward—the lexer just creates a token and moves on.

#### Two-Character Operators (The Lookahead Dance)

Here's where that `peek_char()` method shines! When the lexer sees `=`, it doesn't immediately decide it's an assignment. Instead, it peeks ahead:

```rust
'=' => match self.peek_char() {
    '=' => Token::new(TokenType::Eq, self.read_operator()),      // ==
    '>' => Token::new(TokenType::HashRocket, self.read_operator()), // =>
    _ => Token::new(TokenType::Assign, self.ch.to_string()),     // =
}
```

If the next character is another `=`, we have an equality operator `==`. If it's `>`, we have the hash rocket `=>` (very Ruby-like!). Otherwise, it's just a plain assignment `=`.

The same pattern works for other operators:
- `*` could be multiplication or `**` for exponentiation
- `&` by itself is illegal, but `&&` is the logical AND
- `<` is less-than, but `<=` is less-than-or-equal

#### Strings: Handling the Tricky Bits

Strings are more complex because they span multiple characters and can contain escaped quotes. The lexer has a dedicated `read_string()` method that:

1. Starts when it sees a `"`
2. Keeps reading characters until it hits another `"`
3. Handles escaped quotes `\"` so they don't end the string prematurely
4. Returns an error token if the string isn't properly terminated (unclosed string!)

```rust
// Handle escaped quotes and do not terminate the string prematurely
if self.ch == '\\' && self.peek_char() == '"' {
    self.read_char(); // Skip the escaped quote
    continue;
}
```

#### Identifiers and Keywords

When the lexer encounters a letter or underscore, it reads an entire word using `read_identifier()`. This method keeps consuming characters as long as they're alphanumeric or underscores:

```rust
fn read_identifier(&mut self) -> String {
    let position = self.position;

    while self.ch.is_alphanumeric() || self.ch == '_' {
        self.read_char();
    }

    if SUFFIX_CHARS.contains(&self.ch) {
        self.read_char();  // Handle Ruby-style suffixes like ! and ?
    }

    self.input[position..self.position].iter().collect()
}
```

Once it has the full word, it checks if it's a keyword (like `def`, `if`, `while`, `return`) or just a regular identifier (variable/function name) using the `lookup_identifier()` function.

Topaz also supports Ruby-style method names that end with `!` or `?`, like `empty?` or `save!`. Pretty cool!

#### Numbers: Integers and Floats

Numbers get special treatment too. The lexer needs to distinguish between integers (`42`) and floats (`3.14`), and it also supports underscores for readability (`1_000_000`).

The `read_number()` method:
1. Reads all digits and underscores
2. Checks if there's a decimal point followed by more digits
3. Validates that underscores are used correctly (no consecutive underscores, no trailing underscores)
4. Returns either an Integer or Float token (or Illegal if malformed)

```rust
// Check if we have a decimal point followed by a digit
if self.ch == '.' && Self::is_digit(self.peek_char()) {
    self.read_char(); // consume '.'
    let decimal_part_valid = self.read_number_part();
    // ... determine if it's a valid float
}
```

#### Symbols: The Ruby Touch

Topaz supports Ruby-style symbols (like `:name` or `:error`) and symbol keys (like `name:` in a hash). When the lexer sees a `:`, it calls `read_symbol()` which determines whether it's followed by a valid identifier or if it's malformed.

#### Comments

When the lexer hits a `#`, it knows everything until the end of the line is a comment. It reads all those characters and bundles them into a Comment token:

```rust
fn read_comment(&mut self) -> String {
    let position = self.position;
    self.read_char(); // Move past the '#'

    // Read until end of line or end of file
    while self.ch != '\n' && self.ch != '\0' {
        self.read_char();
    }

    self.input[position..self.position].iter().collect()
}
```

## The Beauty of Iteration

One really neat feature is that the Topaz lexer implements Rust's `Iterator` trait. This means you can use it in for loops and with all of Rust's iterator methods:

```rust
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
```

So you can write clean code like:

```rust
for token in Lexer::new("x = 42") {
    println!("{:?}", token);
}
```

The lexer keeps producing tokens until it hits the end of the file (`EOF`), at which point it returns `None` and the iteration stops.

## Error Handling: When Things Go Wrong

Not all code is valid, and the lexer needs to handle that gracefully. Throughout the implementation, you'll see checks for illegal tokens:

- Identifiers starting with `_` followed by a digit (like `_123`)
- Unterminated strings (missing closing `"`)
- Invalid symbols (`:` followed by numbers or special characters)
- Malformed numbers (consecutive underscores, trailing underscores)
- Single `&` or `|` characters (Topaz requires `&&` and `||`)

When the lexer encounters these problems, it creates an `Illegal` token rather than crashing. This allows error reporting to happen at a higher level in the interpreter.

## The Token Types

The lexer recognizes a rich set of token types:

- **Literals**: Identifier, Integer, Float, String, Symbol
- **Operators**: Assignment (`=`), arithmetic (`+`, `-`, `*`, `/`, `**`), comparison (`==`, `!=`, `<`, `>`, `<=`, `>=`), logical (`&&`, `||`, `!`)
- **Delimiters**: Parentheses, braces, brackets, commas, semicolons, dots
- **Keywords**: `def`, `do`, `end`, `if`, `else`, `elsif`, `while`, `return`, `true`, `false`, `nil`
- **Special**: Comments, EOF, Illegal tokens

Each token is a simple struct containing its type and the actual text (literal) from the source code:

```rust
pub struct Token {
    pub token_type: TokenType,
    pub literal: String,
}
```

## Putting It All Together

Here's what happens when you feed the lexer this code:

```ruby
def greet(name)
  puts "Hello, #{name}!"
end
```

The lexer produces this list of tokens:

1. `Def` ("def")
2. `Identifier` ("greet")
3. `LParen` ("(")
4. `Identifier` ("name")
5. `RParen` (")")
6. `Identifier` ("puts")
7. `String` ("\"Hello, #{name}!\"")
8. `End` ("end")
9. `EOF` ("")

Each token carries the information about what it is and what it looked like in the source code. The parser (the next stage of the interpreter) can then take this clean stream of tokens and build an Abstract Syntax Tree (AST) to understand the structure and meaning of the code.

## Why This Design Works

The Topaz lexer is a great example of good software design:

1. **Simple State**: It only tracks what it needs - the input, current position, and current character
2. **Single Responsibility**: Each method has one job (read a string, read a number, skip whitespace, etc.)
3. **Clear Flow**: The `next_token()` method is easy to understand, with each character case handled explicitly
4. **Robust Error Handling**: Invalid input produces Illegal tokens rather than panics
5. **Idiomatic Rust**: It leverages Rust's pattern matching, iterators, and type system beautifully

## Wrapping Up

Lexing is the foundational step in interpreting or compiling code. It might seem mechanical by just breaking text into chunks, but doing it well requires careful attention to edge cases, good error handling, and thoughtful design. Writing tests alongside the code is invaluable to help you spot when you've introduced a bug or missed an edge case.

The Topaz lexer shows that you don't need complex algorithms or fancy techniques to build something solid. With clear logic, good organization, and thorough testing, you can create a lexer that's both easy to understand and reliable.

Now that we have a lexer in place, I thought I'd see how well it performs on some sample Topaz code. In the next post, I'll share some benchmarks and insights from running the lexer on real code snippets.

Until then! 👋