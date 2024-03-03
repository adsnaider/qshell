#![warn(missing_docs)]
#![warn(missing_debug_implementations)]
#![doc = include_str!("../README.md")]

pub mod qcmd;

pub use qcmd::{QCmd, QCmdBuilder};
pub use sh_macro::cmd;

/// Similar to the lower-level [`cmd`] macro that also executes the commands in order.
#[macro_export]
macro_rules! sh {
    ($($stream:tt)*) => {
        $crate::cmd!($($stream)*)
            .for_each(|cmd| cmd.exec().expect("Command execution failure"));
    };
}
