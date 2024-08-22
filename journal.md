# Tj's TOML Parser Journal 

This is a journal for the purpose of tracking the progress for the TOML parser I aim to build in Rust.

## Questions

Here is a living list of questions I have about the design:

- Q: How am I going to track the table keys already found? Can I create a nested structure that's accessible via the TOML-specified *dot-delimited* keys?

- Q: How am I going to parse tokens that require lookahead (i.e. array of tables' `[[` and `]]` delimiters)
    - A: I do this using the `Peekable` struct.
- Q: How am I going to find comments at the end of values?
    - A: I am currently grouping the current line into segments based on important character delimiters. `#` is one such delimiter, meaning segments beginning with it are either comments or part of a string (determined via context).
- Q: Even assuming the different numeric TOML values are parsable, how do I determine which function to call to begin with?
    - A: I'm currently considering a try-catch type of procedure, passing the current line to each function until a value is successfully parsed.

---

## 22 August 2024

- Added a function to parse the numeric types (int, float, and date). Since there is no direct way of determining which value is present, we have to try all three functions until a match is found (or throw an error otherwise).
- Added test for `parse_numeric`
- Wrote a prototype function for parsing values. Added test, but will need to add on to it once arrays and inline tables are implemented.

## 21 August 2024

Today, I want to take stock of the current parsing functions to ensure I know what is happening in each one. Meaning, 

- What assumption(s) do I make about the parsing context upon entering a given function?
- What information is returned?

### To-Do

At this stage, I now have to complete the following for the parser:

- [ ] Key-Value Parsing
- [ ] Inline Tables 
- [ ] Arrays 
- [ ] Table exportation 
- [ ] Testing
- [ ] Reorganization and public interfacing

## 20 August 2024

There were hidden bugs. One such bug was a nasty infinite loop. I've fixed them, and I've also replaced the comment parsing function with a function that instead processes the end of a given line, comment or no comment. This way, I don't need to do any special checking before parsing a comment; I can just say, "process the rest of the line".

One question I need to answer now is *when* to process the end of a line. I think I may do it at the end of a given parsing function. This way, I don't need to return the context from an upper-level parsing function, and I can assume that the line is completely done with.

... After some thought, the answer is "no". Because I have to take arrays into account, I can't process the end of a given line after parsing a value because there may be a comma at the end of a value. In other words, unlike a table header context, I can't guarantee that there isn't a comma that signifies another element.

## 19 August 2024

I tested and debugged the table heading parsing function. I also implemented the array of tables heading function and tests.

Given the following TOML:

```toml
[  x.y.z.w]
[x.y."\u0001f525"]
[   x         ]   # this is a comment
[[hello.variety]]
[hello.subtable]
[[hello.variety]]
[[hello.variety]]
[[hello]]
[[hello]]
[[hello]]
```

Here is the following (debug formatted) output:
```
{"x": HTable({"y": HTable({"🔥": HTable({}), "z": HTable({"w": HTable({})})})}), "hello": AoT([{"variety": AoT([{}, {}, {}]), "subtable": HTable({})}, {}, {}, {}])}
```
 
It works as desired, even generating subtables within array of tables. This was a great success. Hopefully, no unknown bug will surface.

## 17 August 2024

Wrote the prototype for processing table headers, taking into account the type of table encountered as the table structure is traversed. Not sure if the code is "elegant" or organized properly, but I've commented the relevant function extensively to document what's going on.

## 16 August 2024

I added the test for key parsing, and it worked on the first try! This marks the first Rust code snippet that I was able to implement without logic mistakes. A momentous occasion.

## 15 August 2024

I've been working, but I haven't been journaling. I've come up with a way to represent a key path through the table. Essentially, I replace a given `.` in a dotted key path with a null character `\0`. This is fine because `\0` is an invalid character in TOML, so no information is altered through its use as a delimiter for my own purpose.


Additionally, I've written (but haven't tested) the function to parse keys. I will need to test this function, but that's a job for tomorrow or later.

## 1 August 2024

I think I may have figured out TOML tables, PTMH. The TOML spec states that tables may only be defined once. I think the word "defined" is a bit misleading because, to me, it implies that once a tbale is instantiated, it is immutable. As this is not the case—we need to be able to add additional items to a given table, otherwise, there's not much use in the configuration language,— perhaps a better term is "declared". Even this, however, is an imperfect replacement because supertables can be fully defined after its subtable if the supertable was created as a result of defining the subtable.

### Dotted Keys vs. Table Headers
In any case, the verbiage of the spec was a bit confusing because I had a different impression (an implicit assumption?) regarding tables themselves. I assumed that tables were tables regardless of the syntax used to declare them (with array of tables and inline tables being exceptions). However, it is now clear to me that tables declared via header syntax are a different type than those declared via dotted key syntax *under* a table heading. Meaning,

```toml
# file 1
[fruits.apple]
color = "red"

# ---
# file 2
[fruits]
apple.color = "red"
```

are different types even though the resulting table is the same. If I were to introduce a new `TOMLType` variant for dotted-key-instatiated tables called `DKTable`, then the resulting structures would be the following:

```toml
# file1
# {"fruits": Table({"apple": DKTable({"color": "red"})})}

# ---
#file2
# {"fruits": DKTable({"apple": DKTable({"color": "red"})})}
```

The hashmap is the same strucuturally, but the types are different. Representing these two cases as separate types makes the table header rules much easier to process.
Using the same example, the rule that a table defined by a header cannot be re-defined via dotted key is easily understood:

```toml
[fruits]
apple.color = "red"

[fruits.apple]  # INVALID
```

Since `apple` is initially defined as a `DKTable`, we can't then create a normal `Table`. In fact, the rule would be written as:

**Table Header Dotted Label Rule**
> A dotted table header whose last segment points to a dotted key table is invalid. Dotted headers that *extend* dotted key tables are allowed.

So, continuing the last example, `[fruits.apple]` is invalid because `apple` is defined as a `DKTable`. However, a dotted header that extends `apple` is fine:

```toml
[fruits]
apple.color = "red"

# [fruits.apple]        # BAD
[fruits.apple.texture]  # VALID: `texture` doesn't exist as a key in `apple`, so it can define a subtable of type `Table`.
# ... some stuff
```


### Valid Table Header

Whether a table header is valid depends on what each segment points to. The simple case is obvious: if the table exists already, the new label is invalid.

0. For the dotted case, if you are defining a subtable in terms of uninitialized supertables, the header is valid and all supertables are initialized as empty.
0. If the segments of a dotted string are already initialized **and** each parent segment points to a `Table`, the header is invalid only if it has been used already.
0. If **any** segment of a header points to a `DKTable`, the header is only valid if it *extends* the last `DKTable` in the chain.
0. If the first header segment points to an array of tables (AoT), there must be subsequent segments.
    ```toml
    [[fruits]]          # declares an AoT under "fruits"

    [fruits]            # INVALID: fruits cannot be a `Table` and an `AoT`

    [fruits.banana]     # Valid: declares a `Table` named "banana" under the last table in the "fruits" array.
                        # If the label already exists in said table, of course, this header invalid.
    ```

### Handling Keys

I think I want a way to be able to handle and compare dotted keys via a path-like structure. I wanted something like Rust's `Path`, but there doesn't seem to be a general method of representing a path through a graph structure. Also, I couldn't push the key segments to a `Path` because there's a possibility that a user includes a `/` as part of a key segment, which would split one segment into two.

An idea I was inspired with was creating my own simple structure that can do what I want. As a delimiter, I can use a character that is invalid in TOML such as the null character U+00. This way, I *know* the keys are properly segmented in the custom `Path` type.

**Desire Features**
- Specify the delimiter
- Get the segments of the Path
- Iterable
- Comparable (PathA == PathB?)

I think this type may be useful in storing keys that have already been defined as dotted headers.

## 31 July 2024

Still working to understand how to process tables. It's taking time to fully get a comprehensive perspective on the task at hand. TOML has many stipulations surrounding how tables are to be handled. My goal is to process tables via recursion, but, to do so, I need to take table headers, array of tables, inline tables, and dotted keys all into consideration. For example, the dotted key introduces so much additional logic overhead for me. Specifically, the interaction between dotted key and table headers is confusing.


## 29 July 2024

I was thinking about the design for the representation of the parsed TOML information.
Initially, due to the key-value-based format, I intended to use a HashMap. This would work because I can store all of the TOMLType values into the structure, but I think traversal would be a little...involved. 

When thinking, I realized that I wanted something that would make a given path easily traversable. The use of the term "path" eventually resulted in realizing that I can model the entire structure as a virtual file system. A *directory* is the current "level" of the structure and points to other pairs (*files*). 

A given item has two components: its *name* (key) and its *contents* (value).
The one thing that may be difficult is determining how to implement the recursion.
Maybe have one element contain a `Box<Tree>` if I call the type `Tree`?

## 28 July 2024

Added date and boolean parsing. Modified tests; did some refactoring.

## 27 July 2024

Just for fun, I tried to modify `ParserLine::find_segments` to label the segments semantically. However, I discovered that this is a very difficult task. Because a given delimiter character such as `[` have different functions within different contexts, it's hard to determine what the label should be. This is especially true given that the current line's context cannot be determined when considering multi-line values.

Even more impeding is the fact that some values themselves can contain keys and values (inline tables, arrays [via inline tables]). 

### TOMLSeg

I created a more concrete `TOMLSeg` type. This type contains the segment as a slice of the entire string, allowing for operations such as trimming, replacing, etc. without needing to allocate a String first. It also serves as a casing for the iterator over the graphemes. Since I can now create the graphemes from the slice itself, I don't need to use `Take` or `Skip` to create the iterator. Therefore, it is now of type `Peekable<Graphemes<'a>>`. 

I still need to properly test the new structure (and adjust the code accordingly), but I like this design much more. The bad news is that I can't directly label the segment for types such as dates. As a happy medium, however, because I now store the entire segment as a &str, I can analyze the slice for type-specific characters such as `:` to determine the type, obviating the need to use a brute-force approach.

```rust
struct TOMLSeg<'a> {
    content: &'a str,
    iter: Peekable<Graphemes<'a>>,
}
```

## 25 July 2024

A Reddit comment now has me thinking about how I could have designed my parse tooling to be more effective. Specifically, since I think about and operate over a line in terms of *segments*, I should have considered making a segment type. Well, in fact I did, but it is simply an alias for the iterator type I'm using. What would have been useful, however, is identifying the type of the segment: comment, key, table, string, int, float, date, etc. This may have introduced a more ergonomic parsing design to more easily determine which function to call, especially for types that take similar forms (ints, floats, and dates).

Maybe I could have done something like:

```rust
struct TOMLSeg<'a> {
    content: &'a str,
    label: SegType,
}
impl<'a> TOMLSeg<'a>{
    // functions here
}

enum SegType {
    Int,
    Float,
    Date,  // may split into three sub-categories: NaiveDate, NaiveTime, and DateTime
    Comment, 
    Key,
    Table, 
    InlineTable, 
    String,
    Bool,
    // etcetera
}
```

Since I iterate over the line anyway when creating a `ParserLine` from scratch, I could have a vector of `SegType`s as well...right? Then, iteration over `ParserLine` would produce `TOMLSeg`s which can then produce an iterator `Peekable<Take<Skip<Graphemes<'a>>>>` as necessary.

## 24 July 2024

I prototyped parsing of dates and booleans. For the former, I used the same method as done for floats—get the data in a representation that can then be passed to the external parsing function. For booleans, I avoided this method because I didn't feel it warranted the extra allocation. I'm still exploring the balance between premature optimization and "mature" taste. 

Nevertheless, I still need to implement the prototypes into the `TOMLParser` type. After that, the atomic TOML values should be done (i.e. independently parsable given correct input). After that, there's going to be a lull as I hammer out my design and thinking for handling keys. I have to figure this out first in order to parse arrays and inline tables. This stage is also important for determining how to *access* the data given a successful parsing operation.

Basically, things are about to get real.

## 23 July 2024

Completed integer parsing and tested its functionality. So far, so good. I figured out that I also needed a way to express an empty iterator for the case in which the context ParserLine returns exhausted. I did so by creating a `TOMLSeg<'a>` from the empty `&str`.

In terms of TOML values, I still need to figure out how to parse floats; dates; and the composite values, inline table and array. I don't think the latter can be done, however, until I decide how I will handle keys and the key context. How do I track what keys have been used and what key is in use currently? This will take a lot of thought.

For float parsing, I won't do it by hand because I think handling the precision issues will be a nightmare. Instead, I need to find a way to get the current ParserLine segment into a form that is easily parsable using `str::parse::<f64>()`. Even still, will the method be able to handle TOML-specific float syntax?

### UPDATE

I got float parsing to work, PY. I basically used a shortcut, filtering the grapheme iterator into a String and subsequently calling the `f64` parse function.
I think I may need to consider using the same method for parsing dates via `chrono` if possible. What a fortuitous success!

## 18 July 2024

Major code reorganization. Implemented basic and multi-line string parsing. Literal string parsing comes at a later date.

## 17 July 2024

I think I may have finally stumbled upon my desired struct that allows for the parsing context to be passed around. A major impediment to progress was the inability to pass around the `TOMLSeg<'a>` iterator, especially mid-iteration. To solve this, I modified `ParserLine` to be able to reconstruct the iterator with the necessary offset. 

This may be a more general lesson for Rust. It is likely better to "compress" the data such that, instead of passing along a state, you pass an object that can reconstruct said state. I've written the new implementation below:

```rust
#[derive(Debug)]
pub struct ParserLine {
    line_num: usize,
    data: String,
// iteration things
    seg_nums: Vec<usize>, // a vector of what is essentially cursor positions to denote segment ranges.
    iter_limit: usize,    // The iteration termination value
    curr_seg_num: usize,  // x: 0 <= x <= iter_limit;
    remaining_graphemes: usize, // a tracker for reproducing a given segment with some offset.
}
```

## 16 July 2024

Still working on how to represent the data mid-parsing. After many redesigns--including a venture into creating an iterator object (which did compile, by the way)--the current design has three entities:

- `TOMLParser` - the struct that stores the reader, buffer, and the current line number
- `ParserLine` - a struct that serves as a manual iterator over the current line.
    - Currently, I've decided to make this *practically* an iterator, meaning it doesn't officially implement the trait. This is because I couldn't understand how to get the signature working. Instead, `peek` and `next` (`next_seg` in the `impl`), are regular struct methods.
- `TOMLSeg<'a>` - an alias to a `Peekable<Skip<Take<Graphemes<'a>>>> ` object. This is an iterator over the line's graphemes that represent a TOML-semantic unit. 
    - For example, given a line `key = value #comment\n`, a segment would be an iterator over `#comment\n`. This, in theory, helps with keeping the parsing "clean".

**NOTE**: if possible, I would really enjoy having a means of passing a `TOMLSeg<'a>` through multiple functions as needed.

The time spent on this aspect of the program, while sizeable, has been invaluable for gaining familiarity with lifetimes, referencing rules, and augmenting my understanding of how Rust works. One such example is the use of recursive functions instead of updating important variables in a loop. Passing ownership into a new function call proved to be easier.

## 11 July 2024

I've been creating drafts of various tooling needed for various parsing tasks. Such things include structs for tracking context, processing comments, and processing strings.

In the process, I have become more comfortable with iterators, even being able to create one myself for a custom type. I still need to figure out the best way to structure the code such that the buffer and iterators are cleanly organized.

## 21 June 2024

Another redesign (of course). Instead of allocating a vector for each grapheme poll, I am now using the `unicode_segmentation::Grapheme` iterator instead. I learned that, yes, `Iterator`s do have `skip` and `peek` via *Provided Methods* on Traits.

I created a simple test binary to test my work thus far. At the time of writing, I am able to properly load a TOML file, parse top-level comments, and print debug information.

## 20 June 2024

Still experimenting with the design of the program. Specifically, for the past few days, I've iterated on the representation of the UTF-8 graphemes. I fought the borrow checker valiantly, ultimately succumbing to the fact that my approach was incorrect.
What I wanted was a struct that contained both a `String` representing the current line of the TOML file and a vector of graphemes:

```rust
struct ParseContext<'a> {
    curr_line: String,
    graphemes: Vec<'a>,
    ...
}
```

This is the incorrect approach because the lifetime of `ParseContext` depends on the lifetimes of the references in `graphemes` (which depend on curr_line, but I don't think the compiler knows that). Once graphemes is initialized for the first time, the references within can't be removed--the lifetime annotation promises the references will be valid for at least as long as the `ParseContext` lives.

The only solution I tried to make this work resulted is the entire struct being borrowed for the duration of the lifetime, meaning I could not update it.

### Solution

I don't particularly enjoy the solution, but I've created a factory for the graphemes object. Every time the method is called, a vector of string slices is instantiated and returned to the caller. Since a new vector is created on each call, it will be faithful to updates to `curr_line`. What I find regrettable about this solution is that it results in memory allocation on each instantiation, which could result in considerable impediments to performance.

This will likely be something I try to address as a post-mortem.

## 18 June 2024

I've been thinking of changing how I parse the TOML file. Since TOML is delimited by newline, it makes sense to parse by line, building up the overall hash map throughout the program. The only concern I have so far is how to handle multi-line strings.

I think I should begin thinking of parsing/lexing in terms of contexts. Meaning, depending on what I am attempting to generate, the interpretation of the input varies.

### Testing skipping whitespace

Whitespace is allowed in various places in the TOML format for aesthetic reasons, meaning it must be skipped. The problem is that if I'm iterating using an iterator, I can't find the first non-whitespace character without consuming it. 

That is, until I found the `peek` method on iterators. Today's goal is to learn how to skip whitespace via an iterator using `peek`.

### Results

I was able to get a working implementation of skipping whitespace and processing TOML comments. As it turns out, I have not needed `peek` yet.

In terms of design, I've created a struct that has the necessary context involved in parsing TOML. This includes fields for line number, cursor position, the current line, etc. I figured a single object is easier to pass around than multiple objects. 

However, I'm beginning to think the current struct will just be the entire parser. In my head, I was going to implement the parser as a series of functions that take a mutable reference to a context object. However, said functions may as well just be methods on the object itself.

What I'm learning is that while planning is important, implementation will result in numerous redesigns, especially when dealing with quirks of the implementation language.

## 13 June 2024

```rust
fn main() {
    let mut x: Vec<Token> = Vec::new(); 
    x.push(Token { value: Box::new(1.5), token_type: TokenTypes::Float});
    x.push(Token { value: Box::new("String".to_string()), token_type: TokenTypes::String});

    let y = Vec::from_iter(x.iter_mut().map(|k| -> Box<dyn Debug> { k.get_val() }));  // This results in a tuple!
    x.push(Token{ value: Box::new(y), token_type: TokenTypes::Array });  // in reality, I'd drain the elements that I modified replacing it with the tuple.
    println!("{:?}", x);
}

// Every TOML data type may be represented as a type that implements Debug
// tuple, HashMap, ints, floats, String, and even chrono::DateTime<Tz> if I use that.
use std::fmt::Debug;

#[derive(Debug)]
struct Token {
    value: Box<dyn Debug>,
    token_type: TokenTypes,
}

impl Token {
    /// move the current value out, replacing it with some default.
    fn get_val(&mut self) -> Box<dyn Debug> {
        use std::mem;
        mem::replace(&mut self.value, Box::new("replaced"))
    }
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

```
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
```

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
```
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
```

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

- [x] Load file
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


---

## Parsing Context Assumptions

When calling function X, I assume that we begin:

| Parsing Function              | Context                                                           | 
| ----------------              | -------                                                           |
| **Value Parsing**             | On Whitespace or valid value character                            |
| **String Functions**          |                                                                   | 
| `parse_string`                | On the first `"`                                                  | 
| `parse_multi_string`          | On the first `"`                                                  |
| `parse_basic_string`          | On the first `"`                                                  |
| `parse_multi_escape_sequence` | Immediately *after* the backslash                                 |
| `parse_basic_escape_sequence` | Immediately *after* the backslash                                 |
| `get_nonwhitespace`           | Either on whitespace character OR on exhausted ParserLine         |
| `escape_utf8`                 | Immediately after the `u` in an escaped `\u` sequence             |
| `parse_literal_string`        | On the first `'`                                                  | 
| `parse_multi_string`          | On the first `'`                                                  |
| `parse_basic_litstr`          | On the first `'`                                                  |
| **Integer Parsing**           |                                                                   |
| `parse_integer`               | Assume we start on whitespace or directly on a valid character    |
| `dec_parse`                   | On a digit from 1..=9 (but not 0)                                 |
| `nondec_parse`                | On some digit from 0..=F                                          |
| `hex_parse`                   | On some digit from 0..=F                                          | 
| `oct_parse`                   | On some digit from 0..=7                                          |
| `bin_parse`                   | On some digit from 0..=1                                          |
| **Float Parsing**             |                                                                   |
| `parse_float`                 | Whitespace or on valid float character                            |
| **Boolean Parsing**           |                                                                   |
| `parse_bool`                  | On whitespace or on character.                                    |
| **Date Parsing**              |                                                                   |
| `parse_date`                  | On whitespace or on character.                                    |
| **Array Parsing**             |                                                                   |
| `parse_array`                 | On `[`                                                            |
| **Table/Key Parsing**         |                                                                   |
| `parse_key`                   | On whitespace or on character.                                    |
| `parse_table_header`          | On `[`                                                            |
| `parse_aot_header`            | On Whitespace or valid key character                              |
| `parse_inline_table`          | On `{`                                                            |

