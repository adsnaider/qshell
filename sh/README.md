# `sh`: Command-running macro

This crate provides two macros for facilitating interactions with the underlying system.
The [`cmd`] macro is the lower level macro that implements a DSL to construct
[`QCmd`]s. The [`sh`] macro is a thin wrapper on top that executes each command
in sequence, panicking if there's a failure.

The DSL allows for easily piping data into and out of the commands from [`String`]s and
[`Vec<u8>`]s.

# Examples

```rust
# use sh::sh;
# #[cfg(target_os = "linux")]
# fn run() {
let world = "world";
let mut out = String::new();
// We can use expressions as arguments
// and pipe the cmd output to a String or Vec<u8>
sh!(echo hello {world} > {&mut out});
assert_eq!(out, "hello world\n");

// We can also pipe a String/&str or Vec<u8>/&[u8] to a command
out.clear();
let input = "foo bar baz";
sh!(cat < {input} > {&mut out});
assert_eq!(&out, input);

// We can execute many commands at once
let mut out1 = String::new();
let mut out2 = String::new();
let mut out3 = String::new();

sh! {
  echo hello world 1 > {&mut out1}; // Note the `;`
  echo hello world 2 > {&mut out2};
  echo hello world 3 > {&mut out3};
}

assert_eq!(&out1, "hello world 1\n");
assert_eq!(&out2, "hello world 2\n");
assert_eq!(&out3, "hello world 3\n");
# }
# run();
```

For more information, see the documentation for [`cmd`].

## Future Goals

* Support piping from/to files
* Support piping from one command to the next
