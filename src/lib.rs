//! A command-running macro.
//!
//! `sh` is a macro for running external commands. It provides functionality to
//! pipe the input and output to variables as well as using rust expressions
//! as arguments to the program.
//!
//! # Examples
//!
//! ```
//! # use sh::sh;
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
//! # use sh::sh;
//! # #[cfg(target_os = "linux")]
//! # fn run() {
//! let world = "world";
//! let mut out = String::new();
//! sh!(echo hello {world} > {out});
//! assert_eq!(out, "hello world\n");
//! # }
//! # run();
//! ```
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
