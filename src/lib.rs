//! ctl10n (compile time localization) provides you a simple way to embed messages
//! into binary file without embedding them into source. Internally, ctl10n generates
//! simple `macro_rules!` macro `tr!()` from provided a TOML file with strings.
//!
//! # Basic usage
//! Add ctl10n to your `build-dependencies` in your `Cargo.toml`.
//! If you want to use `include_strings` you'll need it in `dependencies` as well.
//!
//! ```toml
//! [package]
//! name = "example"
//! version = "0.1"
//! edition = "2018"
//!
//! [build-dependencies]
//! ctl10n = "0.1.0"
//!
//! [dependencies]
//! ctl10n = "0.1.0"
//! ```
//!
//! Add the following to your `build.rs`:
//! ```no_run
//! use ctl10n;
//!
//! fn main() {
//!     println!("cargo:rerun-if-changed=build.rs");
//!     println!("cargo:rerun-if-changed=strings.toml");
//!     if let Err(err) = ctl10n::convert_default_strings_file() {
//!         panic!("{}", err);
//!     }
//! }
//! ```
//!
//! This will generate the file `$OUT_DIR/strings.rs` from `strings.toml`.
//! The TOML file with strings must be a table where all values are strings. Example `strings.toml`:
//! ```toml
//! message = "Some message"
//! message-with-args = "Some message with {arg}"
//! ```
//!
//! You should include `strings.rs` somewhere (for example, in `lib.rs`) to use the generated
//! macro. You can do this by calling the macro `ctl10n::include_strings!()` or manually,
//! using `include!()`.
//! After including the macro it can be used like this:
//! ```ignore
//! use ctl10n;
//!
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
//!
//! Output of this code (assuming `strings.toml` from above):
//! ```text
//! Some message
//! Some message with foobar
//! Some message with foobaz
//! ```
//! Trying to use an unknown key or wrong format arguments is a compile-time error.
//!
//! # Multiple locales
//! You can use environment variables to provide a different locale at compile time:
//!
//! ```
//! use ctl10n;
//! use std::env;
//!
//! fn main() {
//!     println!("cargo:rerun-if-changed=build.rs");
//!     println!("cargo:rerun-if-changed=locales/*.toml");
//!     if let Err(err) = ctl10n::convert_strings_file(
//!         format!(
//!             "locales/{}.toml",
//!             &env::var("LOCALE").unwrap_or("en".to_string())
//!         ),
//!         "strings.rs",
//!     ) {
//!         panic!("{}", err);
//!     }
//! }
//! ```
//!
//! `LOCALE=de cargo build`

use std::env;
use std::fmt::Display;
use std::fs;
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
    () => {
        include!(concat!(env!("OUT_DIR"), "/strings.rs"));
    };
    ($filename:tt) => {
        include!(concat!(env!("OUT_DIR"), "/", $filename));
    };
}

/// Convert TOML string to Rust source code with `tr!()` macro
pub fn gen_strings_macro(input: &str) -> Result<String> {
    let strings = parse_toml(input)?;
    let kv: Vec<(&str, &str)> = strings
        .iter()
        .map(|(k, v)| (k.as_ref(), v.as_ref()))
        .collect();
    let keys = kv.iter().map(|(fst, _)| fst);
    let values = kv.iter().map(|(_, snd)| snd);

    let result = quote! {
        macro_rules! ctl10n_tr_inner {
            #( (#keys) => { #values } );*;
            ($key:tt) => {
                compile_error!(concat!("There is no string for key `", stringify!($key), "`"))
            };
        }

        macro_rules! tr {
            ($key:tt) => { ctl10n_tr_inner!($key) };
            ($key:tt, $( $args:tt )* ) => { format!(ctl10n_tr_inner!($key), $( $args )* ) };
        }
    };
    Ok(result.to_string())
}

/// Convert given TOML file to Rust source code in given location, providing
/// macro `tr!()`
pub fn convert_strings_file(
    toml_file: impl AsRef<Path> + Display,
    rs_file: impl AsRef<Path>,
) -> Result<()> {
    let mut input_file = fs::File::open(toml_file)?;
    let mut input = String::new();
    input_file.read_to_string(&mut input)?;
    let code = gen_strings_macro(&input)?;
    let mut output_file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(rs_file)?;
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
        Path::new(&env::var("OUT_DIR").unwrap()).join("strings.rs"),
    )
}
