//! ctl10n (compile time localization) provides you a simple way to embed messages
//! into binary file without embedding them into source. Internally, ctl10n generates
//! simple `macro_rules!` macro `tr!()` from provided TOML file with strings.
//! # Basic usage
//! Put following to your `build.rs`:
//! ```no_run
//! fn main() {
//!     println!("cargo:rerun-if-changed:build.rs");
//!     println!("cargo:rerun-if-chaged:strings.toml");
//!     if let Err(err) = ctl10n::convert_default_strings_file() {
//!         panic!("{}", err);
//!     }
//! }
//! ```
//! This will generate file `$OUT_DIR/strings.rs` from `strings.toml`.
//! TOML file with strings must be table where all values are strings. Example `strings.toml`:
//! ```toml
//! message = "Some message"
//! message-with-args = "Some message with {arg}"
//! ```
//! You should include `strings.rs` somewhere (for example, in `lib.rs`) to use generated
//! macro. You can do this by calling macro `ctl10n::include_strings!()` or manually,
//! using `include!()`.
//! After including macro it can be used like this:
//! ```ignore
//! ctl10n::include_strings!();
//!
//! fn main() {
//!     // `tr!()` with one argument will be translated to string literal
//!     println!(tr!("message"));
//!     println!(tr!("message-with-args"), arg = "foobar");
//!     // `tr!()` with multiple arguments will be translated to formatted `&String`
//!     println!("{}", tr!("message-with-args", arg = "foobaz"))
//! }
//! ```
//! Output of this code (assuming `strings.toml` from above):
//! ```text
//! Some message
//! Some message with foobar
//! Some message with foobaz
//! ```
//! Trying to use unknown key or wrong format arguments is compile-time error.


use std::fs;
use std::env;
use std::fmt::Display;
use std::io::{Read, Write};
use std::path::Path;

use quote::quote;

mod error;
pub use crate::error::{Error, Result};

mod toml_parser;
use toml_parser::parse_toml;

/// Include `tr!()` macro from generated file to current namespace.
/// If called without arguments includes file `$OUT_DIR/strings.rs`.
/// If called with one argument includes corresponding file in `$OUT_DIR`.
#[macro_export]
macro_rules! include_strings {
    () => { include!(concat!(env!("OUT_DIR"), "/strings.rs")); };
    ($filename:tt) => { include!(concat!(env!("OUT_DIR"), "/", $filename)); };
}

/// Convert TOML string to Rust source code with `tr!()` macro
pub fn gen_strings_macro(input: &str) -> Result<String> {
    let strings = parse_toml(input)?;
    let kv: Vec<(&str, &str)> = strings.iter().map(|(k, v)| (k.as_ref(), v.as_ref())).collect();
    let keys = kv.iter().map(|(fst, _)| fst);
    let values = kv.iter().map(|(_, snd)| snd);

    let result = quote! {
        macro_rules! clt10n_tr_inner {
            #( (#keys) => { #values } );*;
            ($key:tt) => {
                compile_error!(concat!("There is no string for key `", stringify!($key), "`"))
            };
        }

        macro_rules! tr {
            ($key:tt) => { clt10n_tr_inner!($key) };
            ($key:tt, $( $args:tt )* ) => { &format!(clt10n_tr_inner!($key), $( $args )* ) };
        }
    };
    Ok(result.to_string())
}

/// Convert given TOML file to Rust source code in given location, providing
/// macro `tr!()`
pub fn convert_strings_file(toml_file: impl AsRef<Path> + Display, rs_file: impl AsRef<Path>) -> Result<()> {
    let mut input_file = fs::File::open(toml_file)?;
    let mut input = String::new();
    input_file.read_to_string(&mut input)?;
    let code = gen_strings_macro(&input)?;
    let mut output_file = fs::OpenOptions::new().write(true).create(true).open(rs_file)?;
    output_file.write(&code.as_bytes())?;
    Ok(())
}

/// Convert file `strings.toml` in current diretory to file `strings.rs` in `$OUT_DIR`
/// # Panics
/// If environment variable `OUT_DIR` is not set. You should call this function only
/// from `build.rs` script
pub fn convert_default_strings_file() -> Result<()> {
    convert_strings_file(
        "strings.toml",
        Path::new(&env::var("OUT_DIR").unwrap()).join("strings.rs")
    )
}
