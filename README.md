# Topaz

A programming language implementation in Rust.

## Lexer

Consider implementing:

#### Literals

1. Being able to create a float like this .5 (leading zero optional)
2. Being able to create a float like this 5. (trailing zero optional)
3. Being able to create differnt kinds of numbers eg hexadecimal, binary, octal
4. Being able to create numbers with exponents eg 1.5e10,

## Development

### Running the Project

```bash
cargo run
```

### Running Tests

```bash
cargo test
```

### Running Benchmarks

The project includes comprehensive benchmarks for the lexer to measure performance across different scenarios.

**Run all benchmarks:**

```bash
cargo bench
```

**Run specific benchmark:**

```bash
cargo bench --bench lexer_benchmark -- "small program"
```

**Available benchmarks:**
- `small program` - Basic variable assignments and function calls
- `medium program` - Recursive functions and array operations
- `complex program` - Comprehensive test with hashes, arrays, symbols, and nested structures
- `operators heavy` - Focus on mathematical and logical operators
- `string heavy` - Tests string tokenization performance

**View results:**

After running benchmarks, detailed HTML reports with graphs and statistics are generated in:
```
target/criterion/
```

Open the `index.html` file in your browser to view visual performance reports.
