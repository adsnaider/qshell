#![doc = include_str!("../../README.md")]
//!
//! # Examples
//!
//! ```
//! # use qshell::sh;
//! # #[cfg(target_os = "linux")]
//! # fn run() {
//! let world = "world";
//! let mut out = String::new();
//! sh!(echo hello {world} > {out});
//! assert_eq!(out, "hello world\n");
//! # }
//! # run();
//! ```
//!
//! ```
//! # use qshell::sh;
//! # #[cfg(target_os = "linux")]
//! # fn run() {
//! let world = "world";
//! let mut out = String::new();
//! sh!(echo hello {world} > {out});
//! assert_eq!(out, "hello world\n");
//! # }
//! # run();
//! ```
//!
//! For more information, see the documentation for
//! [`sh`].
pub use sh_macro::sh;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn echo() {
        sh! {
            echo hello world
        };
        let mut out = String::new();
        sh!(echo hello world > {out});
        assert_eq!(out, "hello world\n");
    }

    #[test]
    fn multiple() {
        // sh! {
        // echo hello world;
        // echo hello world;
        // }
    }
}
