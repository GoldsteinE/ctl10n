# ctl10n

ctl10n (compile time localization) provides you a simple way to embed messages
into binary file without embedding them into source. Internally, ctl10n generates
a simple `macro_rules!` macro `tr!()` from the provided a TOML file with strings.

## Basic usage
Add ctl10n to your `build-dependencies` in your `Cargo.toml`.
If you want to use `include_strings` you'll need it in `dependencies` as well.

```toml
[package]
name = "example"
version = "0.1"
edition = "2018"

[build-dependencies]
ctl10n = "0.1.0"

[dependencies]
ctl10n = "0.1.0"
```

Add the following to your `build.rs`:
```rust
use ctl10n;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=strings.toml");
    if let Err(err) = ctl10n::convert_default_strings_file() {
        panic!("{}", err);
    }
}
```

This will generate the file `$OUT_DIR/strings.rs` from `strings.toml`.
The TOML file with strings must be a table where all values are strings. Example `strings.toml`:
```toml
message = "Some message"
message-with-args = "Some message with {arg}"
```

You should include `strings.rs` somewhere (for example, in `lib.rs`) to use the generated
macro. You can do this by calling the macro `ctl10n::include_strings!()` or manually,
using `include!()`.
After including the macro it can be used like this:
```rust
use ctl10n;

ctl10n::include_strings!();

fn main() {
    // `tr!()` with one argument will be translated to string literal
    println!(tr!("message"));
    println!(tr!("message-with-args"), arg = "foobar");
    // `tr!()` with multiple arguments will be translated to formatted `&String`
    println!("{}", tr!("message-with-args", arg = "foobaz"))
}
```

Output of this code (assuming `strings.toml` from above):
```
Some message
Some message with foobar
Some message with foobaz
```
Trying to use an unknown key or wrong format arguments is a compile-time error.

## Multiple locales
You can use environment variables to provide a different locale at compile time:

```rust
use ctl10n;
use std::env;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=locales/*.toml");
    if let Err(err) = ctl10n::convert_strings_file(
        format!(
            "locales/{}.toml",
            &env::var("LOCALE").unwrap_or("en".to_string())
        ),
        "strings.rs",
    ) {
        panic!("{}", err);
    }
}
```

`LOCALE=de cargo build`
