# TOML Parser - My First Rust Project

In order to gain familiarity in a new programming language as well as improve my parsing and project design abilities, I set out to implement a parser for the [TOML](https://toml.io) markup language. I chose TOML because both Python and Rust—the language I know and the one I want to learn—use the format, and it seemed to be both approachable enough to be doable and complex enough to stretch my abilities. The goal was to implement the parser myself without extensive use of other crates (though I did use a couple). This encouraged me to dive into the standard library, allowing me to learn the language basics without developing an over-reliance on external tools. 

This repository is the result. It includes:

- A function `tomlp::parse` that parses TOML and returns a viewable table, `tomlp::ParsedTOML`.
- `ParsedTOML` implements `std::fmt::Display`, resulting in a print out that emulates the format of the `tree` program for file systems.
- The type is a wrapper around `tomlp::TOMLTable`, an alias for a hash map. I created a trait, `tomlp::ValFromTOMLKey` that allows the user to input a multi-part key with a user-specified delimiter, subsequently querying the table and returning the result as an `Option<tomlp::TOMLType>`. 

## Example

I wanted to see if the parser worked on a "real" TOML file, so I used the `Cargo.toml` from the [`ripgrep`](https://github.com/BurntSushi/ripgrep) project. Assuming the file is named `ripgrep.toml` and that it's in the current working direectory:

```rust
fn main() -> Result<(), String> {
    use tomlp::{parse, ValFromTOMLKey};

    let result = parse("ripgrep.toml")?;
    println!("Parsed TOML Table:{}", result);

    // Query the table.
    // Let's get an array
    println!("\nRetrieved Value:\n{:?}", result.retrieve("package\0keywords", "\0"));
    Ok(())
}
```

The above program results in the following output:

```
Parsed TOML Table:
/
├── workspace
│   └── members
│       └── ARRAY
├── test (Arr_of_Tbls)
│   ├── name
│   │   └── integration
│   └── path
│       └── tests/tests.rs
├── package
│   ├── authors
│   │   └── ARRAY
│   ├── keywords
│   │   └── ARRAY
│   ├── edition
│   │   └── 2021
│   ├── build
│   │   └── build.rs
│   ├── version
│   │   └── 14.1.0
│   ├── documentation
│   │   └── https://github.com/BurntSushi/ripgrep
│   ├── license
│   │   └── Unlicense OR MIT
│   ├── description
│   │   └── MULTI-LINE STRING
│   ├── rust-version
│   │   └── 1.72
│   ├── metadata
│   │   └── deb
│   │       ├── assets
│   │       │   └── ARRAY
│   │       ├── features
│   │       │   └── ARRAY
│   │       ├── section
│   │       │   └── utils
│   │       └── extended-description
│   │           └── MULTI-LINE STRING
│   ├── exclude
│   │   └── ARRAY
│   ├── autotests
│   │   └── false
│   ├── repository
│   │   └── https://github.com/BurntSushi/ripgrep
│   ├── categories
│   │   └── ARRAY
│   ├── homepage
│   │   └── https://github.com/BurntSushi/ripgrep
│   └── name
│       └── ripgrep
├── target
│   └── cfg(all(target_env = "musl", target_pointer_width = "64"))
│       └── dependencies
│           └── jemallocator
│               └── version
│                   └── 0.5.0
├── dependencies
│   ├── log
│   │   └── 0.4.5
│   ├── bstr
│   │   └── 1.7.0
│   ├── termcolor
│   │   └── 1.1.0
│   ├── lexopt
│   │   └── 0.3.0
│   ├── textwrap
│   │   ├── default-features
│   │   │   └── false
│   │   └── version
│   │       └── 0.16.0
│   ├── grep
│   │   ├── version
│   │   │   └── 0.3.1
│   │   └── path
│   │       └── crates/grep
│   ├── serde_json
│   │   └── 1.0.23
│   ├── anyhow
│   │   └── 1.0.75
│   └── ignore
│       ├── version
│       │   └── 0.4.22
│       └── path
│           └── crates/ignore
├── dev-dependencies
│   ├── walkdir
│   │   └── 2
│   ├── serde
│   │   └── 1.0.77
│   └── serde_derive
│       └── 1.0.77
├── bin (Arr_of_Tbls)
│   ├── name
│   │   └── rg
│   ├── bench
│   │   └── false
│   └── path
│       └── crates/core/main.rs
├── features
│   └── pcre2
│       └── ARRAY
└── profile
    ├── release-lto
    │   ├── inherits
    │   │   └── release
    │   ├── panic
    │   │   └── abort
    │   ├── incremental
    │   │   └── false
    │   ├── opt-level
    │   │   └── 3
    │   ├── codegen-units
    │   │   └── 1
    │   ├── debug-assertions
    │   │   └── false
    │   ├── strip
    │   │   └── symbols
    │   ├── overflow-checks
    │   │   └── false
    │   ├── debug
    │   │   └── none
    │   └── lto
    │       └── fat
    ├── deb
    │   ├── debug
    │   │   └── false
    │   └── inherits
    │       └── release
    └── release
        └── debug
            └── 1

Retrieved Value:
Some(Array([BasicStr("regex"), BasicStr("grep"), BasicStr("egrep"), BasicStr("search"), BasicStr("pattern")]))
```

I'm really happy about how this project turned out.

## Lessons Learned

Coming from Python, I was very accustomed to mutation via self-references, so the borrow checker was interesting to contend with. At one point in the project, I realized that instead of passing state directly, something the Rust compiler didn't accept because of how I attempted it, I could instead structure my parsing tools to record, pass, and then recreate the context between function calls. I'm not sure it was the most "idiomatic" way to achieve my goal, but that wasn't a priority for this project in the first place. 

Secondly, I was surprised by how much I enjoyed working with iterators. I really like that the methods defined in the `Iterator` trait allowed me to work with iterators without regard to what Item was inside. The MVP for this project was the `Peekable` struct by far. Being able to look ahead was crucial, at least with how my thinking was oriented during the process.

Lifetimes were another thing I had to understand. I'm still not all the way there, but I'm now able to reason about them such that I can debug lifetime errors relatively quickly. I think implementing the `Iterator` trait on a custom type that held a `&str` was a major force in understanding the concept since I had to confront it directly. Jon Gjengset's [archived livestream](https://www.youtube.com/watch?v=rAl-9HwD858) on lifetimes was also incredibly helpful in addition to numerous articles and online posts. 

A major lesson learned in this project centered around typing. In the early stages of the project, I couldn't wrap my head around storing values of different types in one data structure. The solution, I learned, was to use indirection, representing each type as a tuple enum variant (the `TOMLType` enum). This allowed for all of the different types to be considered as one higher-level type. I also learned about *monomorphization* and *dynamic dispatch* as well as their trade offs (major thanks to the programmers on the [Rust subreddit](https://reddit.com/r/rust) for their explanations of these concepts).

Perhaps the most fun part of the project for me was seeing how quickly higher-level parsing operations such as parsing arrays, inline tables, and entire tables were developed. I was able to leverage the lower-level functions to make the higher-level operations simple to write. It was also really cool to be able to define types that represented my mental model of the problem and subsequently create my solution in terms of said types.

Tooling-wise, I also made an effort to learn Vim and Git by reading their respective documentation and practicing. Setting up rust-analyzer was initially a challenge, but it was fine once I had a better grasp of Vim plugins. Still, I did use VSCode from time-to-time. 

One aspect of the project that could have been better was error handling. I knew going into this project that I didn't know enough about it, so, considering that, I'm satisfied with how it turned out. Honestly, diving into proper error handling in addition to all of the other concepts would have been overwhelming and likely would have significantly impeded my progress and decreased my motivation. However, as I improve, I want to learn how errors are handled properly, especially in large applications (I don't think propagating a String upwards would cut it).

Overall, this project provided the opportunity to grow as a software designer, a programmer, and a project manager. There was also personal growth, as I made relatively frequent use of online forums (Reddit), something I am normally reluctant to do. One thing I learned a while ago is that it's okay to ask for guidance, especially after genuinely contending with a given problem. I learned very quickly from the explanations provided by more knowledgable Rust programmers.  

I'm enjoying my time with Rust, and I hope to learn more in the future in domains I'm experienced with as well as those I've yet to touch such as asynchronous/concurrent programming.

\- tjdwill
