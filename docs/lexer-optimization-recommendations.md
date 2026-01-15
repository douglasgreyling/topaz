# Lexer Performance Optimization Recommendations

## Overview
This document provides performance optimization recommendations for the Topaz lexer implementation based on analysis of the current codebase.

## Critical Performance Issues

### 1. **String Slicing Overhead in Single-Character Tokens**
**Impact: High** | **Difficulty: Easy**

**Current Issue:**
Every single-character token creates a new string slice like `&self.input[self.position..self.position + 1]`. This is repeated ~15 times in `next_token()`.

```rust
')' => Token::new(
    TokenType::RParen,
    &self.input[self.position..self.position + 1],
),
```

**Recommendation:**
Pre-compute and reuse string slices for single-character tokens, or better yet, use static string references:
```rust
// At module level
const LPAREN_STR: &str = "(";
const RPAREN_STR: &str = ")";
// etc.

// In next_token():
'(' => Token::new(TokenType::LParen, LPAREN_STR),
')' => Token::new(TokenType::RParen, RPAREN_STR),
```

**Alternative:** Cache the slice at the beginning of `next_token()`:
```rust
let single_char_slice = &self.input[self.position..self.position + 1];
```

**Expected Improvement:** 5-10% faster tokenization for operator-heavy code.

---

### 2. **Keyword Lookup Uses Linear String Matching**
**Impact: Medium** | **Difficulty: Medium**

**Current Issue:**
The `lookup_identifier()` function performs linear pattern matching on every identifier. For programs with many identifiers, this creates unnecessary overhead.

```rust
pub fn lookup_identifier(ident: &str) -> TokenType {
    match ident {
        "def" => TokenType::Def,
        "do" => TokenType::Do,
        // ... 11 comparisons total
        _ => TokenType::Identifier,
    }
}
```

**Recommendation:**
Use a perfect hash function or `phf` crate for O(1) keyword lookups:
```rust
use phf::phf_map;

static KEYWORDS: phf::Map<&'static str, TokenType> = phf_map! {
    "def" => TokenType::Def,
    "do" => TokenType::Do,
    "end" => TokenType::End,
    // ... etc
};

pub fn lookup_identifier(ident: &str) -> TokenType {
    KEYWORDS.get(ident).copied().unwrap_or(TokenType::Identifier)
}
```

**Expected Improvement:** 10-15% faster for identifier-heavy code.

---

### 3. **Redundant Bounds Checking in Character Reading**
**Impact: Medium** | **Difficulty: Medium**

**Current Issue:**
Every call to `read_ascii_char()` and `peek_ascii_char()` checks bounds:
```rust
pub fn read_ascii_char(&mut self) {
    if self.read_position >= self.input.len() { // bounds check
        self.ch = '\0';
    } else {
        self.ch = self.input.as_bytes()[self.read_position] as char; // another bounds check
    }
    // ...
}
```

**Recommendation:**
Use `get_unchecked()` after the manual bounds check to eliminate redundant compiler checks:
```rust
pub fn read_ascii_char(&mut self) {
    if self.read_position >= self.input.len() {
        self.ch = '\0';
    } else {
        // SAFETY: We just checked the bounds above
        self.ch = unsafe { *self.input.as_bytes().get_unchecked(self.read_position) as char };
    }
    self.position = self.read_position;
    self.read_position += 1;
}
```

**Alternative:** Store `input.len()` as a field to avoid repeated `.len()` calls.

**Expected Improvement:** 3-5% overall performance gain.

---

### 4. **Inefficient Comment Reading**
**Impact: Low-Medium** | **Difficulty: Easy**

**Current Issue:**
The `read_comment()` function creates a `chars()` iterator and manually advances the lexer:
```rust
fn read_comment(&mut self) -> &'a str {
    let start = self.position;
    let remaining = &self.input[self.position..];
    let mut chars = remaining.chars();
    
    while let Some(ch) = chars.next() {
        if ch == '\n' { break; }
        self.read_unicode_char();
    }
    &self.input[start..self.position]
}
```

**Recommendation:**
Use `memchr` crate or `str::find()` for faster newline search:
```rust
fn read_comment(&mut self) -> &'a str {
    let start = self.position;
    let remaining = &self.input[self.position..];
    
    if let Some(newline_pos) = remaining.find('\n') {
        self.position += newline_pos;
        self.read_position = self.position + 1;
        self.ch = '\n';
    } else {
        // Comment extends to EOF
        self.position = self.input.len();
        self.read_position = self.input.len();
        self.ch = '\0';
    }
    
    &self.input[start..self.position]
}
```

**Expected Improvement:** 20-30% faster comment processing.

---

### 5. **Whitespace Skipping Uses Array Contains**
**Impact: Low** | **Difficulty: Easy**

**Current Issue:**
```rust
const WHITESPACE_CHARS: [char; 4] = [' ', '\t', '\n', '\r'];

fn skip_whitespace(&mut self) {
    while WHITESPACE_CHARS.contains(&self.ch) { // O(4) linear search per character
        self.read_ascii_char();
    }
}
```

**Recommendation:**
Use a direct character comparison or bitmap for faster checking:
```rust
#[inline]
fn is_whitespace(ch: char) -> bool {
    matches!(ch, ' ' | '\t' | '\n' | '\r')
}

fn skip_whitespace(&mut self) {
    while Self::is_whitespace(self.ch) {
        self.read_ascii_char();
    }
}
```

**Alternative:** For ASCII-only optimization, use a lookup table:
```rust
static WHITESPACE_TABLE: [bool; 128] = {
    let mut table = [false; 128];
    table[b' ' as usize] = true;
    table[b'\t' as usize] = true;
    table[b'\n' as usize] = true;
    table[b'\r' as usize] = true;
    table
};

#[inline]
fn is_whitespace(ch: char) -> bool {
    ch.is_ascii() && WHITESPACE_TABLE[ch as usize]
}
```

**Expected Improvement:** 5-8% faster for whitespace-heavy code.

---

## Moderate Performance Improvements

### 6. **Token Allocation Strategy**
**Impact: Low-Medium** | **Difficulty: Medium**

**Current Issue:**
Each call to `next_token()` returns a new `Token` struct by value. While this is lightweight, we could reduce copying overhead.

**Recommendation:**
Consider using a token stream buffer that reuses allocations:
```rust
pub struct TokenStream<'a> {
    lexer: Lexer<'a>,
    buffer: Vec<Token<'a>>,
}

impl<'a> TokenStream<'a> {
    pub fn tokenize_all(&mut self) {
        self.buffer.clear();
        while let Some(token) = self.lexer.next() {
            self.buffer.push(token);
        }
    }
}
```

**Expected Improvement:** 2-5% for full tokenization scenarios.

---

### 7. **Eliminate Double UTF-8 Validation**
**Impact: Low** | **Difficulty: Hard**

**Current Issue:**
When reading unicode characters, we call `.chars().next()` which validates UTF-8, but the input string is already guaranteed to be valid UTF-8.

**Recommendation:**
Only relevant if profiling shows `read_unicode_char()` is a hotspot. Could use unsafe UTF-8 decoding with `str::as_bytes()` and manual decoding.

**Expected Improvement:** 5-10% for unicode-heavy inputs (probably not worth it for typical code).

---

## Micro-Optimizations

### 8. **Inline Hot Functions**
**Impact: Low** | **Difficulty: Easy**

**Recommendation:**
Add `#[inline]` to frequently called methods:
```rust
#[inline]
fn is_digit(ch: char) -> bool {
    ch.is_digit(10)
}

#[inline]
fn is_whitespace(ch: char) -> bool {
    matches!(ch, ' ' | '\t' | '\n' | '\r')
}

#[inline]
pub fn peek_ascii_char(&self) -> char {
    if self.read_position >= self.input.len() {
        '\0'
    } else {
        self.input.as_bytes()[self.read_position] as char
    }
}
```

**Expected Improvement:** 1-3% overall.

---

### 9. **Reduce Method Call Overhead in Number Reading**
**Impact: Low** | **Difficulty: Medium**

**Current Issue:**
`read_number()` calls `read_number_part()` which has multiple validation checks. The function is called twice for floats.

**Recommendation:**
Consider inlining the logic or restructuring to reduce call overhead:
```rust
#[inline(always)]
fn read_number_part(&mut self) -> bool {
    // ... existing logic
}
```

**Expected Improvement:** 2-4% for number-heavy code.

---

### 10. **String Scanning Optimization**
**Impact: Low** | **Difficulty: Medium**

**Current Issue:**
String reading calls `read_unicode_char()` for every character, which is necessary for unicode support but slow.

**Recommendation:**
For ASCII-only strings (which are common), use a fast path:
```rust
fn read_string(&mut self) -> (TokenType, &'a str) {
    let position = self.position;
    
    // Fast path: scan ASCII bytes directly
    let bytes = self.input.as_bytes();
    let mut i = self.read_position;
    
    while i < bytes.len() {
        let b = bytes[i];
        if b == b'"' {
            self.position = i;
            self.read_position = i + 1;
            self.ch = '"';
            return (TokenType::String, &self.input[position..i+1]);
        } else if b == b'\\' {
            i += 2; // skip escaped char
        } else if b >= 128 {
            // Non-ASCII, fall back to unicode path
            return self.read_string_unicode();
        } else {
            i += 1;
        }
    }
    
    // Unterminated string
    self.position = i;
    self.read_position = i;
    self.ch = '\0';
    (TokenType::Illegal, &self.input[position..])
}
```

**Expected Improvement:** 15-25% faster for string-heavy code.

---

## Architecture Considerations

### 11. **Consider Zero-Copy Token Design**
**Impact: High** | **Difficulty: High**

**Current State:** Tokens already use `&'a str` references (zero-copy). This is already optimal.

**Note:** The current design is excellent for memory efficiency.

---

### 12. **Batch Token Processing**
**Impact: Variable** | **Difficulty: High**

**Recommendation:**
If the parser can benefit from it, consider tokenizing in batches:
- Pre-scan for line boundaries
- Tokenize multiple tokens before yielding
- Use SIMD for whitespace/comment detection

**Note:** This is a significant rewrite and should only be done if profiling shows it's necessary.

---

## Priority Implementation Order

1. **High Priority (Implement First):**
   - String slicing optimization (#1)
   - Keyword lookup optimization (#2)
   - Comment reading optimization (#4)

2. **Medium Priority:**
   - Whitespace optimization (#5)
   - Bounds checking optimization (#3)
   - Inline hot functions (#8)

3. **Low Priority (Measure First):**
   - Token allocation strategy (#6)
   - String scanning optimization (#10)
   - Number reading optimization (#9)

4. **Consider Only If Profiled:**
   - UTF-8 validation (#7)
   - Batch processing (#12)

---

## Benchmarking Recommendations

Before implementing optimizations:
1. Run benchmarks with `cargo bench` to establish baseline
2. Use `cargo flamegraph` to identify actual hotspots
3. Implement optimizations incrementally
4. Re-benchmark after each change
5. Focus on optimizations that show >5% improvement

The current benchmark suite covers good ground. Consider adding:
- Unicode-heavy input benchmark
- Comment-heavy input benchmark
- Very large input benchmark (1MB+)

---

## Code Quality Notes

**Current Strengths:**
- Clean, readable code
- Good separation of concerns
- Comprehensive test coverage
- Zero-copy token design
- Proper lifetime management

**Areas for Improvement:**
- Some functions are quite long (`next_token()` could be split)
- Error handling could use custom error types
- Consider adding source position tracking for better error messages

---

*Last Updated: 2026-01-13*
