# Tj's TOML Parser Journal 

This is a journal for the purpose of tracking the progress for the TOML parser I aim to build in Rust.

## 13 June 2024

```rust
fn main() {
    let mut x: Vec<Token> = Vec::new(); 
    x.push(Token { value: Box::new(1.5), token_type: TokenTypes::Float});
    x.push(Token { value: Box::new("String".to_string()), token_type: TokenTypes::String});

    let y = Vec::from_iter(x.iter().map(|k| -> &Box<dyn Debug> { &k.value }));
    //x.push(Token{ value: Box::new(y), token_type: TokenTypes::Array });
    println!("{:?}, {:?}", x, y);
}

// Every TOML data type may be represented as a type that implements Debug
// tuple, HashMap, ints, floats, String, and even chrono::DateTime<Tz> if I use that.
use std::fmt::Debug;

#[derive(Debug)]
struct Token {
    value: Box<dyn Debug>,
    token_type: TokenTypes,
}

#[derive(Debug)]
enum TokenTypes {
    // Examples
    Float,
    String,
    Array,
    // ...
}
```

## 12 June 2024

Today, I struggled with heterogeneity in Rust. Since TOML has the array type that can store homogeneous data, I need something that can turn into a Rust `tuple` in order to represent heterogeneously-typed data. Some ideas are:

- Dynamic dispatch via `&dyn Trait` 
- An enum with many variants (unwieldy).

## 11 June 2024

I've been working on this project intermittently while also going through Rustlings, but I have not been updating the journal. This will be a catch-up entry.

### Practice

First, I have programmed two practice programs with the aim of improving my parsing skills as well as my familiarity with Rust. The first program was one that takes a Rust source file as input and removes all top-level comments. Among others, I learned about File I/O as well as simple string iteration. The second program was a program that implements `atol`, a function that takes a numeric string and converts it to an integer. Keeping with the theme of lexxing and parsing, I created multiple token types and used them to tokenize the input and subsequently parse it, generating helpful error messages along the way. 

It was through the second program that I've begun to develop a methodology for parsing. For example, creating an invalid token type to make it easy to recognize input errors, having an outer token type that composes the other, more granular tokens and coordinates lexxing, and establishing an order-of-development are all aspects of parsing learned in the latter program. Specifically, I've decided that it makes sense to first get the input properly tokenized, then correctly parse the ideal case (completely valid input), and, finally, work with errors, generating error messages along the way.

One thing I have yet to get a handle on is proper error handling. I want a method that propagates error messages upward, keeping track of line number among other information. More thought will need to go into needed logging information.

Finally, today I worked to figure out the look ahead issue. Specifically, when tokenizing the TOML input, I want to turn input such as `"""` direcly into a triple quote token. Why? Currently, the idea is to push each generated token to the token container vector. Over multiple passes, the vector will need to be recreated as the tokens are processed into higher-level tokens. Parsing the terminal tokens directly saves an interation cycle. I will need to look into vector methods that would make modifying the contents memory efficient.

### Lexxing Overview

I've decided on an initial process for lexxing. Again, I plan to do multiple passes over the input and subsequently-generated tokens to produce higher-order tokens. Said tokens will be more readily parsable according to the TOML grammar. Here's the proposed process, beginning from the raw input:

0. Produce the terminal (primitive) tokens.
    - These are tokens such as comment signals or string quotes. 
1. Retokenize to produce `string` tokens
2. Tokenize again to produce `tablehead` tokens (distinguishing between table heads and arrays).
3. I think that may be all that's needed.

The most important development was step 1. Strings need to be found first because they are sources of ambiguity for interpreting the other tokens. For example, `#` is a comment delimiter, but within a string context, it is instead just a character. This same thing applies to pretty much all other tokens that trigger some TOML grammar function. In other words, within a string context, the other semantic tokens are rendered nonfunctional. Finding the strings first will then segment the token bin effectively into `{strings, non-strings}` such that we know for sure that any `#` token delimits a comment (for example). The same reasoning applies to step 2. Arrays and table heads both begin with `[`, so finding table heads will then make it clear that all other `[` are indeed array delimiters. In practice, this reasoning may require tweaking, but I think it's a great starting point.

### Look ahead

I propose three components. First, we have the outer lexer. This component is responsible for processing the input and generating tokens for the bin. However, there will be cases in which some "inner" workers need to produce the token. These inner workers are those such as the string delimiter tokens. The idea is to pass a mutable reference to the token bin as well as a shared reference to the remaining input vector slice to find the correct token via lookahead. Within this process, a function is called to find the relevant substring for proper tokenization.

+---------------------------------------------------+
| Lexer                                             |
|       +-------------------------------------------|
|       | StringQuote                               |
|       |                                           |
|       |                                           |
|       |  +----------------------------------------|
|       |  | SubString Search                       |
|       |  |                                        |
|       |  |                                        |
|       |  |                                        |
|       |  +----------------------------------------|
|       +-------------------------------------------|
+---------------------------------------------------+


## 22 May 2024

Perhaps I should add an intermediate token level for basic symbols. Things like 

## 21 May 2024

A lot of brainstorming done today. The goal was to spend time thinking about the lexer, specifically what type of tokens to generate. I have decided that, being that this is my first attempt at a formidable parser, perfection is not the goal. I'm going to make mistakes that may require redesigning from zero. That's fine though; that's the point of an *educational* project.

### Lexer

As a design aid, I am allowing myself to load the entire source file into memory instead of worrying about batching. Ignoring that for now, the lexer design that makes the most sense to me is a multi-pass lexer. Given the source file, this is my envisioned process:

1. Raw file -> Character Tokens
    - Character Tokens are tokens that, in addition to identifying numeric and general text data, classify significant characters such as `#`, `[`, `]`, `"` ,`'`, and the like.
2. Character Tokens -> Base Grammar Tokens
    - Base Grammar Tokens: The Basic Units comprising the TOML grammar (ex. Comments, Strings, Ints, Dates, etc.)
3. Base Grammar -> High-Level Tokens (TOML Tokens?)
    - These tokens are the upper-level, more abstracted tokens intended to be processed by the parser directly. These would include concepts such as keys, values, key-value pairs, Table headers, etc.)
    - Still ironing this level out

    +------------------+
    |                  | 
    |  Raw Text File   | 
    |                  | 
    +------------------+
              |
              |
              |
              V
    +------------------+
    |                  | 
    | Character Token  | 
    |                  | 
    +------------------+
              |
              |
              |
              V
    +------------------+
    |                  | 
    |Base Grammar Token| 
    |                  | 
    +------------------+
              |
              |
              |
              V
    +------------------+
    |                  | 
    |    TOML Token    | 
    |                  | 
    +------------------+


That's the tentative plan in terms of lexing. The token level of a given TOML constitutional construct may change. 

### Interface Design

I think ironing out the desired user interface features focuses the API design, so I devised a list.

- `new` Parse TOML (via file handle or path string)
- `get`: return view of parsed table structure.
- `get_keys`: Structure of table keys
- `print`: Print entire structure
- `print_keys`: Print keys (Ideally in nested order)
- `write_json`: Output to JSON format
- `write_toml`: Output to TOML format
    - This is mostly just to reorganize the original TOML file, consolidating the data under relevant tables.
- `dump`: Rust serialization file
- `load`: deserializes a Rust TOML structure from file.

The above would be the User API (Public API). The developer (Private) API will likely be much more involved.

### Tasks

Things I need to do:

- [ ] Load file
- [ ] Generate tokens
- [ ] Write Parser Functions
- [ ] Assemble Table Structure
- [ ] Implement Public API
- [ ] Test Extensively
- [ ] CLI?

## 20 May 2024

Here are a few design questions

1. What will the User API look like?
2. How do I decide what tokens to generate in terms of specificity?
3. One pass or muliple passes?
4. Data persistence? serialization/deserialization?
5. What are the basics of UTF-8?
6. Basics of Augmented Backus-Naur Form (ABNF)?

### Augmented Backus-Naur Form (ABNF)

I learned the basics of ABNF by reading [RFC 5234](https://www.rfc-editor.org/rfc/rfc5234). It's so short that it's worth just reading instead of summarizing it here.


