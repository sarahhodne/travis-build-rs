#![macro_escape]

use std::path::Path;
use std::ops;

#[deriving(Clone)]
pub enum Statement {
    Statements(Box<Vec<Statement>>),
    Fold(String, Box<Statement>),
    Cmd(Command, Vec<CommandOption>),
    If(Condition, Box<Statement>, Box<Statement>),
    Noop
}

#[deriving(Clone, Show)]
pub enum CommandOption {
    /// Print out the command before running it.
    EchoOption,

    /// Fail the script if the command didn't succeed.
    AssertOption,

    /// The string to print before the command. Only makes sense with EchoOption. By default this is `$ the-command`.
    DisplayOption(String)
}

#[deriving(Clone)]
pub enum Condition {
    Exists(Path),
    IsDirectory(Path),
    IsFile(Path),
    CmdCond(Command),
    And(Box<Condition>, Box<Condition>),
    Or(Box<Condition>, Box<Condition>),
    Not(Box<Condition>),
}

#[deriving(Clone)]
pub enum Command {
    Raw(String),
    Echo(String),
    Newline,
    Envset(String, String),
    Cd(Path),
    Putfile(Path, Vec<u8>),
    Mkdir(Path),
    Copyfile(Path, Path),
    Movefile(Path, Path),
    Removefile(Path),
}

#[macro_export]
macro_rules! ast_if (
    ($cond:expr { $($body:expr;)* }) => (::ast::If($cond, box ast_block! { $($body);* }, box ::ast::Noop));
    ($cond:expr { $($body:expr;)* } else { $($elsebody:expr;)* }) => (::ast::If($cond, box ast_block!{ $($body);* }, box ast_block! { $($elsebody);* }))
)

#[macro_export]
macro_rules! format_cmd(
        ([$($opts:ident)|*], $($arg:tt)*) => (::ast::Cmd(::ast::Raw(format!($($arg)*)), vec![$(::ast::$opts),*]));
        ($($arg:tt)*) => (::ast::Cmd(::ast::Raw(format!($($arg)*)), vec![]))
)

#[macro_export]
macro_rules! cmd(
    ([$($opts:ident)|*], $cmd:expr) => (::ast::Cmd($cmd, vec![$(::ast::$opts),*]));
    ($cmd:expr) => (::ast::Cmd($cmd, vec![]))
)

#[macro_export]
macro_rules! ast_set(
    ($key:ident = $value:expr) => (cmd!(ast::Envset(stringify!($key).to_string(), $value)));
)

/// Create a statement that contains the contained statements.
///
/// # Example
///
/// ```ignore
/// let statement = ast_block! {
///     format_cmd!("echo -n hello");
///     format_cmd!("echo world");
/// };
///
/// assert_eq!(ast::Statements(box vec![format_cmd!("echo -n hello"), format_cmd!("echo world")]), statement);
/// ```
#[macro_export]
macro_rules! ast_block(
    { $($x:expr);* } => (::ast::Statements(box vec![$($x),*]));
    { $($x:expr;)* } => (ast_block! { $($x);* })
)

impl Statement {
    pub fn is_noop(&self) -> bool {
        match *self {
            Noop => true,
            _ => false
        }
    }
}

impl ops::Not<Condition> for Condition {
    fn not(&self) -> Condition {
        let clone = (*self).clone();
        Not(box clone)
    }
}
