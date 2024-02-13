use proc_macro2::{Group, Ident, Literal, TokenStream, TokenTree};
use quote::{quote, ToTokens, TokenStreamExt};

enum Arg {
    Literal(String),
    Expr(TokenStream),
}

impl ToTokens for Arg {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Arg::Literal(s) => tokens.append(Literal::string(s)),
            Arg::Expr(e) => tokens.append_all(e.into_token_stream()),
        }
    }
}

enum ParseState {
    Cmd,
    Args,
    SetSink,
    DoneSetSink,
    SetSource,
    DoneSetSource,
}

enum Sink {
    File(String),
    Var(Ident),
}

enum Source {
    File(String),
    Var(Ident),
}

struct ShParser {
    state: ParseState,
    cmd: Option<String>,
    args: Vec<Arg>,
    sink: Option<Sink>,
    source: Option<Source>,
}

struct Sh {
    cmd: String,
    args: Vec<Arg>,
    sink: Option<Sink>,
    source: Option<Source>,
}

#[derive(Debug)]
enum ShTokenTree {
    Value(String),
    EndOfLine,
    Expr(Group),
    Sink,
    Source,
}

impl From<TokenTree> for ShTokenTree {
    fn from(value: TokenTree) -> Self {
        match value {
            TokenTree::Group(g) => ShTokenTree::Expr(g),
            TokenTree::Ident(value) => ShTokenTree::Value(value.to_string()),
            TokenTree::Punct(c) if c.as_char() == ';' => ShTokenTree::EndOfLine,
            TokenTree::Punct(c) if c.as_char() == '>' => ShTokenTree::Sink,
            TokenTree::Punct(c) if c.as_char() == '<' => ShTokenTree::Source,
            TokenTree::Punct(c) => panic!("Unexpected token: {c}"),
            TokenTree::Literal(value) => {
                let literal = litrs::Literal::from(value);
                let value = match literal {
                    litrs::Literal::Bool(b) => b.to_string(),
                    litrs::Literal::Integer(i) => i.to_string(),
                    litrs::Literal::Float(f) => f.to_string(),
                    litrs::Literal::Char(c) => {
                        unimplemented!("Character literals are not implemented")
                    }
                    litrs::Literal::String(s) => s.into_value().into_owned(),
                    litrs::Literal::Byte(_b) => unimplemented!("Byte literals are not implemented"),
                    litrs::Literal::ByteString(_s) => {
                        unimplemented!("Byte literals are not implemented")
                    }
                };
                ShTokenTree::Value(value)
            }
        }
    }
}

#[must_use]
enum ParseResult {
    KeepGoing,
    Done(Sh),
}

impl Default for ShParser {
    fn default() -> Self {
        Self::new()
    }
}

impl ShParser {
    pub fn new() -> Self {
        Self {
            state: ParseState::Cmd,
            cmd: None,
            args: Vec::new(),
            sink: None,
            source: None,
        }
    }

    pub fn feed(&mut self, token: TokenTree) -> ParseResult {
        let token = ShTokenTree::from(token);
        match self.state {
            ParseState::Cmd => {
                let ShTokenTree::Value(value) = token else {
                    panic!("Unexpected command: {token:?}");
                };
                self.cmd = Some(value);
                self.state = ParseState::Args;
            }
            ParseState::Args => match token {
                ShTokenTree::Value(v) => self.args.push(Arg::Literal(v)),
                ShTokenTree::EndOfLine => return ParseResult::Done(self.into_sh()),
                ShTokenTree::Expr(g) => self.args.push(Arg::Expr(g.stream())),
                ShTokenTree::Sink => self.state = ParseState::SetSink,
                ShTokenTree::Source => self.state = ParseState::SetSource,
            },
            ParseState::SetSink => {
                assert!(self.sink.is_none(), "Can't set the sink more than once");
                match token {
                    ShTokenTree::Value(v) => self.sink = Some(Sink::File(v)),
                    ShTokenTree::Expr(g) => {
                        let mut tokens: Vec<_> = g.stream().into_iter().collect();
                        assert_eq!(tokens.len(), 1, "Expected identifier");
                        let token = tokens.pop().unwrap();
                        let TokenTree::Ident(ident) = token else {
                            panic!("Expected identifier. Got {token:?}");
                        };
                        self.sink = Some(Sink::Var(ident));
                    }
                    other => panic!("Unexpected token: {other:?}"),
                }
                self.state = ParseState::DoneSetSink;
            }
            ParseState::SetSource => {
                assert!(self.source.is_none(), "Can't set the source more than once");
                match token {
                    ShTokenTree::Value(v) => self.source = Some(Source::File(v)),
                    ShTokenTree::Expr(g) => {
                        let mut tokens: Vec<_> = g.stream().into_iter().collect();
                        assert_eq!(tokens.len(), 1, "Expected identifier");
                        let token = tokens.pop().unwrap();
                        let TokenTree::Ident(ident) = token else {
                            panic!("Expected identifier. Got {token:?}");
                        };
                        self.source = Some(Source::Var(ident));
                    }
                    other => panic!("Unexpected token: {other:?}"),
                }
                self.state = ParseState::DoneSetSource;
            }
            ParseState::DoneSetSink => match token {
                ShTokenTree::EndOfLine => return ParseResult::Done(self.into_sh()),
                ShTokenTree::Source => self.state = ParseState::SetSource,
                other => panic!("Unexpected token: {other:?}"),
            },
            ParseState::DoneSetSource => match token {
                ShTokenTree::EndOfLine => return ParseResult::Done(self.into_sh()),
                ShTokenTree::Source => self.state = ParseState::SetSource,
                other => panic!("Unexpected token: {other:?}"),
            },
        }
        ParseResult::KeepGoing
    }

    fn finish(mut self) -> Option<Sh> {
        if self.cmd.is_some() {
            Some(self.into_sh())
        } else {
            None
        }
    }

    fn into_sh(&mut self) -> Sh {
        let mut parser = std::mem::take(self);
        Sh {
            cmd: parser.cmd.take().expect("Missing command"),
            args: parser.args,
            sink: parser.sink,
            source: parser.source,
        }
    }
}

/// A command-running macro.
///
/// `sh` is a macro for running external commands. It provides functionality to
/// pipe the input and output to variables as well as using rust expressions
/// as arguments to the program.
///
/// The format of an `sh` call is like so:
///
/// ```ignore
/// sh!( [prog] [arg]* [> {outvar}]? [< {invar}]? [;]? )
/// ```
///
/// Or you can run multiple commands on a single block
///
/// ```ignore
/// sh! {
///   [prog] [arg]* [> {outvar}]? [< {invar}]? ;
///   [prog] [arg]* [> {outvar}]? [< {invar}]? ;
///   [prog] [arg]* [> {outvar}]? [< {invar}]? [;]?
/// }
/// ```
///
/// Arguments are allowed to take the form of identifiers (i.e. plain text),
/// literals (numbers, quoted strings, characters, etc.), or rust expressions
/// delimited by braces.
///
/// # Examples
///
/// ```
/// # use sh_macro::sh;
/// # #[cfg(target_os = "linux")]
/// # fn run() {
/// let world = "world";
/// let mut out = String::new();
/// sh!(echo hello {world} > {out});
/// assert_eq!(out, "hello world\n");
/// # }
/// # run();
/// ```
///
/// ```no_run
/// # use sh_macro::sh;
/// # #[cfg(target_os = "linux")]
/// # fn run() {
/// sh! {
///   echo hello;
///   sleep 5;
///   echo world;
/// }; // prints hello, waits 5 seconds, prints world.
/// # }
/// # run();
/// ```
///
/// You can also use string literals as needed
///
/// ```
/// # use sh_macro::sh;
/// # #[cfg(target_os = "linux")]
/// # fn run() {
/// let mut out = String::new();
/// sh!(echo "hello world" > {out});
/// assert_eq!(out, "hello world\n");
/// # }
/// # run();
/// ```
///
/// # Panics
///
/// * When the command can't be spawned
/// * When there is an error writing into the command pipe
/// * When the output pipe can't be decoded as a UTF-8 string.
#[proc_macro]
pub fn sh(stream: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let stream: TokenStream = stream.into();
    let mut stream = stream.into_iter();
    let mut cmds: Vec<Sh> = Vec::new();

    let mut parser = ShParser::new();
    while let Some(token) = stream.next() {
        match parser.feed(token) {
            ParseResult::KeepGoing => {}
            ParseResult::Done(sh) => cmds.push(sh),
        }
    }
    if let Some(sh) = parser.finish() {
        cmds.push(sh);
    }

    quote! {{

        #(
            {
               #cmds
            }
        )*
    };}
    .into()
}

impl ToTokens for Sh {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let cmd = &self.cmd;
        let args = &self.args;
        tokens.append_all(quote! {
            use std::process::{Command, Stdio, ChildStdin, ChildStdout};
            use std::io::{Read, Write};
            let mut cmd = Command::new(#cmd);
            #(
                cmd.arg(#args);
            )*
            let mut source: Option<&String> = None;
            let mut sink: Option<&mut String> = None;
        });
        let sink = &self.sink;
        match sink {
            Some(Sink::File(_)) => {
                unimplemented!("Writing sh output to file is not yet implemented")
            }
            Some(Sink::Var(ident)) => tokens.append_all(quote! {
                cmd.stdout(Stdio::piped());
                #ident.clear();
                sink = Some(&mut #ident);
            }),
            None => {}
        }
        let source = &self.source;
        match source {
            Some(Source::File(_)) => {
                unimplemented!("Reading sh input from file is not yet implemented");
            }
            Some(Source::Var(ident)) => tokens.append_all(quote! {
                cmd.stdin(Stdio::piped());
                source = Some(&mut #ident);
            }),
            None => {}
        }
        tokens.append_all(quote! {
            let mut child = cmd.spawn().expect("Couldn't start command");
            if let Some(source) = source {
                let mut stdin = child.stdin.take().unwrap();
                stdin.write_all(source.as_bytes()).expect("IO Error writing to pipe");
            }
            let status = child.wait().expect("Child process wasn't running?");
            assert!(status.success());
            if let Some(sink) = sink {
                let mut stdout = child.stdout.take().unwrap();
                stdout.read_to_string(sink).expect("Non-UTF8 string");
            }
        })
    }
}
