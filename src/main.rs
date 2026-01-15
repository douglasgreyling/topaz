use topaz::lexer::Lexer;

fn main() {
    let input = r##"
        # This is a comment
        def hello_world(name) do
          puts "Hello, #{name}!"
        end

        # Mathematical operations
        1 + 2 * (3 - 4) / 5
        2 ** 3  # Power operator

        # String
        "This is a string"

        # Float
        4.0

        # Symbol
        :my_symbol

        # Boolean logic
        true && false
        true || false
        !true

        # Comparisons
        x == y
        x != y
        x >= 10

        # Hash syntax
        { name: "John", age => 30, "height" => 5.9 }

        # Method call
        object.method_name

        if x > 5 && y < 10 do
          result = x + y
        end
    "##;

    for token in Lexer::new(input) {
        println!("{:?}", token);
    }
}
