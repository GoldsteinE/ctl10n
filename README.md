# ctl10n

ctl10n (compile time localization) provides you a simple way to embed messages
into binary file without embedding them into source. Internally, ctl10n generates
simple `macro_rules!` macro `tr!()` from provided TOML file with strings.
## Basic usage
Put following to your `build.rs`:
```rust
fn main() {
    println!("cargo:rerun-if-changed:build.rs");
    println!("cargo:rerun-if-chaged:strings.toml");
    if let Err(err) = ctl10n::convert_default_strings_file() {
        panic!("{}", err);
    }
}
```
This will generate file `$OUT_DIR/strings.rs` from `strings.toml`.
TOML file with strings must be table where all values are strings. Example `strings.toml`:
```toml
message = "Some message"
message-with-args = "Some message with {arg}"
```
You should include `strings.rs` somewhere (for example, in `lib.rs`) to use generated
macro. You can do this by calling macro `ctl10n::include_strings!()` or manually,
using `include!()`.
After including macro it can be used like this:
```rust
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
Trying to use unknown key or wrong format arguments is compile-time error.
