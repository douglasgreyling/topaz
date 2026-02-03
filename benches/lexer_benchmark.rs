use criterion::{black_box, criterion_group, criterion_main, Criterion};
use topaz::lexer::Lexer;
use topaz::token::TokenType;

fn tokenize_small_input(input: &str) {
    let mut lexer = Lexer::new(input);
    while lexer.next_token().token_type != TokenType::EOF {
        // Consume all tokens
    }
}

fn benchmark_small_program(c: &mut Criterion) {
    let input = r#"
        x = 5
        y = 10
        def add(a, b) do
          a + b
        end
        result = add(x, y)
    "#;

    c.bench_function("lexer: small program", |b| {
        b.iter(|| tokenize_small_input(black_box(input)))
    });
}

fn benchmark_medium_program(c: &mut Criterion) {
    let input = r#"
        def fibonacci(n) do
          if n == 0 do
            return 0
          end

          if n == 1 do
            return 1
          end

          return fibonacci(n - 1) + fibonacci(n - 2)
        end

        result = fibonacci(10)

        def map(arr, f) do
          if arr.length == 0 do
            []
          else
            [f(arr.first)] + map(arr.rest, f)
          end
        end

        numbers = [1, 2, 3, 4, 5]
        doubled = map(numbers, def(x) do x * 2 end)
    "#;

    c.bench_function("lexer: medium program", |b| {
        b.iter(|| tokenize_small_input(black_box(input)))
    });
}

fn benchmark_complex_program(c: &mut Criterion) {
    let input = r#"
        # This is a complex program with many tokens
        calculator = {
            add => def(a, b) do a + b end,
            subtract => def(a, b) do a - b end,
            multiply => def(a, b) do a * b end,
            divide => def(a, b) do a / b end,
            power => def(a, b) do a ** b end
        }

        test_strings = [
            "hello world",
            "foo bar baz",
            "the quick brown fox",
            "jumps over the lazy dog"
        ]

        def operators_test() do
            a = 10
            b = 20

            a == b
            a != b
            a < b
            a > b
            a <= b
            a >= b

            true && false
            true || false
            !true
        end

        def array_ops() do
            arr = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
            filtered = filter(arr, def(x) do x > 5 end)
            mapped = map(filtered, def(x) do x * x end)
            reduced = reduce(mapped, 0, def(acc, x) do acc + x end)
            reduced
        end

        def hash_ops() do
            person = {
                name => "John Doe",
                age => 30,
                "email" => "john@example.com",
                address => {
                    "street" => "123 Main St",
                    "city" => "Anytown",
                    "zip" => "12345"
                }
            }
            person.name
        end

        def symbol_test() do
            key1 = :atom
            key2 = :another_atom
            hash = {atom: "value1", another_atom: "value2"}
            hash
        end
    "#;

    c.bench_function("lexer: complex program", |b| {
        b.iter(|| tokenize_small_input(black_box(input)))
    });
}

fn benchmark_operators_heavy(c: &mut Criterion) {
    let input = r#"
        1 + 2 - 3 * 4 / 5 ** 6
        (1 + 2) * (3 - 4) / (5 + 6)
        a == b && c != d || e < f && g > h
        x <= y && a >= b || !z
        (a + b) * (c - d) == (e / f) ** (g + h)
    "#;

    c.bench_function("lexer: operators heavy", |b| {
        b.iter(|| tokenize_small_input(black_box(input)))
    });
}

fn benchmark_string_heavy(c: &mut Criterion) {
    let input = r#"
        "string one"
        "string two with more content"
        "string three with even more content here"
        "the quick brown fox jumps over the lazy dog"
        "lorem ipsum dolor sit amet consectetur adipiscing elit"
        "multiple" + "strings" + "concatenated" + "together"
    "#;

    c.bench_function("lexer: string heavy", |b| {
        b.iter(|| tokenize_small_input(black_box(input)))
    });
}

criterion_group!(
    benches,
    benchmark_small_program,
    benchmark_medium_program,
    benchmark_complex_program,
    benchmark_operators_heavy,
    benchmark_string_heavy
);
criterion_main!(benches);
