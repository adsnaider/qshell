#![doc = include_str!("../README.md")]
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
#[cfg(target_os = "linux")]
mod tests {
    use super::*;

    #[test]
    fn echo() {
        sh!(echo hello world);
        let mut out = String::new();
        sh!(echo hello world > {out});
        assert_eq!(out, "hello world\n");
    }

    #[test]
    fn multiple() {
        sh! {
            echo hello world;
            echo hello world;
        };
    }

    #[test]
    fn literals() {
        let mut out = String::new();
        sh!(echo "hello world" > {out});
        assert_eq!(out, "hello world\n");

        sh!(echo 1 2 3u8 > {out});
        assert_eq!(out, "1 2 3u8\n");

        sh!(echo 1.23 > {out});
        assert_eq!(out, "1.23\n");

        sh!(echo true > {out});
        assert_eq!(out, "true\n");

        sh!(echo 'a' 'b' > {out});
        assert_eq!(out, "a b\n");
    }
}
