#![warn(missing_docs)]
#![warn(missing_debug_implementations)]
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
//! sh!(echo hello {world} > {&mut out});
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
//! sh!(echo hello {world} > {&mut out});
//! assert_eq!(out, "hello world\n");
//! # }
//! # run();
//! ```
//!
//! For more information, see the documentation for
//! [`cmd`].
pub mod qcmd;

pub use qcmd::{QCmd, QCmdBuilder};
pub use sh_macro::cmd;

/// Similar to the lower-level [`cmd`] macro that also executes the commands in order.
#[macro_export]
macro_rules! sh {
    ($($stream:tt)*) => {
        sh_macro::cmd!($($stream)*)
            .into_iter()
            .for_each(|cmd| cmd.exec().unwrap());
    };
}
