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

struct Cmd {
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
                    litrs::Literal::Char(_c) => {
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
    Done(Cmd),
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

    fn finish(mut self) -> Option<Cmd> {
        if self.cmd.is_some() {
            Some(self.into_sh())
        } else {
            None
        }
    }

    fn into_sh(&mut self) -> Cmd {
        let mut parser = std::mem::take(self);
        Cmd {
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
/// The format of a `cmd` call is like so:
///
/// ```ignore
/// cmd!( [prog] [arg]* [> {outvar}]? [< {invar}]? [;]? )
/// ```
///
/// Or you can create multiple commands on a single block
///
/// ```ignore
/// cmd! {
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
/// This macro doesn't execute the commands. It returns a vector of [`qcmd::QCmd`] which
/// can be executed with. Alternatively, see `qshell::sh` to do the execution for you.
///
/// # Examples
///
/// ```
/// # use sh_macro::cmd;
/// # #[cfg(target_os = "linux")]
/// # fn run() {
/// let world = "world";
/// let mut out = String::new();
/// cmd!(echo hello {world} > {out}).into_iter().for_each(|cmd| cmd.exec().unwrap());
/// assert_eq!(out, "hello world\n");
/// # }
/// # run();
/// ```
///
/// ```no_run
/// # use sh_macro::cmd;
/// # #[cfg(target_os = "linux")]
/// # fn run() {
/// cmd! {
///   echo hello;
///   sleep 5;
///   echo world;
/// }.into_iter().for_each(|cmd| cmd.exec().unwrap()); // prints hello, waits 5 seconds, prints world.
/// # }
/// # run();
/// ```
///
/// You can also use string literals as needed
///
/// ```
/// # use sh_macro::cmd;
/// # #[cfg(target_os = "linux")]
/// # fn run() {
/// let mut out = String::new();
/// cmd!(echo "hello world" > {out}).into_iter().for_each(|cmd| cmd.exec().unwrap());
/// assert_eq!(out, "hello world\n");
/// # }
/// # run();
/// ```
#[proc_macro]
pub fn cmd(stream: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let stream: TokenStream = stream.into();
    let mut stream = stream.into_iter();
    let mut cmds: Vec<Cmd> = Vec::new();

    let mut parser = ShParser::new();
    while let Some(token) = stream.next() {
        match parser.feed(token) {
            ParseResult::KeepGoing => {}
            ParseResult::Done(sh) => cmds.push(sh),
        }
    }
    if let Some(cmd) = parser.finish() {
        cmds.push(cmd);
    }

    quote!(
        {
            let mut commands = Vec::new();
            #(
                commands.push({
                    #cmds
                });
            )*
            commands
        }
    )
    .into()
}

impl ToTokens for Cmd {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let cmd = &self.cmd;
        let args = &self.args;
        tokens.append_all(quote! {
            use std::process::Command;
            let mut cmd = Command::new(#cmd);
            #(
                cmd.arg(#args);
            )*
            let mut source: Option<&str> = None;
            let mut sink: Option<&mut String> = None;
        });
        let sink = &self.sink;
        match sink {
            Some(Sink::File(_)) => {
                unimplemented!("Writing command output to file is not yet implemented")
            }
            Some(Sink::Var(ident)) => tokens.append_all(quote! {
                #ident.clear();
                sink = Some(&mut #ident);
            }),
            None => {}
        }
        let source = &self.source;
        match source {
            Some(Source::File(_)) => {
                unimplemented!("Reading command input from file is not yet implemented");
            }
            Some(Source::Var(ident)) => tokens.append_all(quote! {
                source = Some(&#ident);
            }),
            None => {}
        }
        tokens.append_all(quote! {
            qcmd::QCmd::new(cmd, source, sink)
        })
    }
}
