+++
date = '2026-01-15T12:34:03+02:00'
draft = false
title = 'Hello, Topaz!'
tags = ['announcement']
+++

Hi!

My name is Doug, and this is a hobby project where I'll be documenting my journey creating a new programming language called Topaz.

### What exactly is Topaz?

For now it's a dynamically typed language inspired by Ruby. Hence the name Topaz (a precious stone, like Ruby).

### Why am I doing this?

For fun. I want to learn how programming languages work. I don't think I'll be able to make something serious, but this also provides an opportunity to
learn, experiment, and play with technologies which I don't get to use in my day job.

### What will Topaz look like?

For the most part it will look just like Ruby with some differences here and there. I had the wonderful opportunity to learn some Elixir a few years back
and I may include some small Elixir-like features as well.

Onto the syntax!

```
# This is a comment
def greet(name) do
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
```

As you can see this is very similar to Ruby with some Elixir influences (like the `do` and `end` keywords for blocks).

I believe strongly that code is read far more than it is written, and as a result I want Topaz to be a language that is easy to read and understand.

Having some inspiration behind the language provides me with an opportunity to peek under the hood of these languages and learn how they work. I can draw on the patterns and ideas that have already been tried and tested.

### What're you going to build it with?

I'm taking this opportunity to learn Rust, so the compiler and tooling etc will be built with Rust.

This'll mean that the implementation may be a little clunky at times whilst I learn how Rust works, and that's fine. The goal is mostly to have opportunities to learn new things, and nothing stops me from coming back and refining things once I know how to do it better.

### What's next?

In the long term, Topaz will be a functionly programming lanuage when:

1) It has a working lexer
2) It has a working parser
3) It has a working interpreter

The good news is that I've already got a working implementation of a lexer. I've had a chance to sink my teeth into some Rust, and I think I've learnt a lot already. I'll be writing about my first implementation of the lexer soon, and then I'll probably follow that up with some lessons which I learnt on making it faster and more performant.

Stay tuned!
