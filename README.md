# `sh`: Command-running macro

`sh` is a macro for running external commands. It provides functionality to
pipe the input and output to variables as well as using rust expressions
as arguments to the program.

`cmd` is the lower-level macro, returning a QCmd vector that can be manually
spawned.
