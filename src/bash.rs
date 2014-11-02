use ast;
use serialize::base64;
use serialize::base64::ToBase64;

pub fn shellescape(input: &str) -> String {
    if input.is_empty() {
        return "''".to_string();
    }

    let mut output = String::with_capacity(input.len());

    for ch in input.chars() {
        if ch >= 'A' && ch <= 'Z' {
            output.push(ch);
        } else if ch >= 'a' && ch <= 'z' {
            output.push(ch);
        } else if ch >= '0' && ch <= '9' {
            output.push(ch);
        } else if ['_', '-', '.', ',', ':', '/', '@'].contains(&ch) {
            output.push(ch);
        } else if ch == '\n' {
            output.push_str("'\n'");
        } else {
            output.push('\\');
            output.push(ch);
        }
    }

    output
}

fn indent(input: &str) -> String {
    let mut result = String::new();
    let mut iter = input.split('\n').map(|s| format!("  {}", s) );
    let mut first = true;

    for s in iter {
        if first {
            first = false;
        } else {
            result.push('\n');
        }
        result.push_str(s.as_slice());
    }

    result
}

pub trait ToBash {
    fn to_bash(&self) -> String;
}

impl ToBash for ast::Statement {
    fn to_bash(&self) -> String {
        match self {
            &ast::Statements(ref stmts) => {
                let mut result = String::new();
                let mut iter = stmts.iter().filter(|s| !s.is_noop()).map(|stmt| stmt.to_bash());
                let mut first = true;

                for s in iter {
                    if first {
                        first = false;
                    } else {
                        result.push_str("\n");
                    }
                    result.push_str(s.as_slice());
                }
                result
            },
            &ast::Fold(ref fold_name, ref stmt) => format!("travis_fold start {0}\n{1}\ntravis_fold end {0}", fold_name, stmt.to_bash()),
            &ast::Cmd(ref command, ref options) => {
                let mut options_str = String::new();
                for option in options.iter() {
                    match *option {
                        ast::EchoOption => options_str.push_str(" --echo"),
                        ast::AssertOption => options_str.push_str(" --assert"),
                        ast::DisplayOption(ref display) => options_str.push_str(format!(" --display={}", shellescape(display.as_slice())).as_slice()),
                    }
                }

                format!("travis_cmd {}{}", shellescape(command.to_bash().as_slice()), options_str)
            },
            &ast::If(ref condition, ref body, ref elsebody) => {
                match **elsebody {
                    ast::Noop => format!("if {}; then\n{}\nfi", condition.to_bash(), indent(body.to_bash().as_slice())),
                    ast::If(_, _, _) => format!("if {}; then\n{}\nel{}", condition.to_bash(), indent(body.to_bash().as_slice()), indent(elsebody.to_bash().as_slice())),
                    _ => format!("if {}; then\n{}\nelse\n{}\nfi", condition.to_bash(), indent(body.to_bash().as_slice()), indent(elsebody.to_bash().as_slice()))
                }
            },
            &ast::Noop => "".to_string()
        }
    }
}

impl ToBash for ast::Command {
    fn to_bash(&self) -> String {
        match *self {
            ast::Raw(ref cond) => cond.clone(),
            ast::Echo(ref string) => format!("echo {}", string),
            ast::Newline => "echo".to_string(),
            ast::Envset(ref var, ref value) => format!("export {}={}", var, shellescape(value.as_slice())),
            ast::Cd(ref path) => format!("cd {}", shellescape(path.as_str().unwrap())),
            ast::Putfile(ref path, ref contents) => {
                let path_str = path.as_str().unwrap();
                let base64_body = contents.as_slice().to_base64(base64::STANDARD);
                format!("base64 --decode > {} <<<{}", shellescape(path_str), shellescape(base64_body.as_slice()))
            },
            ast::Mkdir(ref path) => format!("mkdir -p {}", shellescape(path.as_str().unwrap())),
            ast::Copyfile(ref from_path, ref to_path) => format!("cp -r {} {}", shellescape(from_path.as_str().unwrap()), shellescape(to_path.as_str().unwrap())),
            ast::Movefile(ref from_path, ref to_path) => format!("mv {} {}", shellescape(from_path.as_str().unwrap()), shellescape(to_path.as_str().unwrap())),
            ast::Removefile(ref path) => format!("rm -rf {}", shellescape(path.as_str().unwrap())),
        }
    }
}

impl ToBash for ast::Condition {
    fn to_bash(&self) -> String {
        match *self {
            ast::Exists(ref path) => format!("[[ -e {} ]]", shellescape(path.as_str().unwrap())),
            ast::IsDirectory(ref path) => format!("[[ -d {} ]]", shellescape(path.as_str().unwrap())),
            ast::IsFile(ref path) => format!("[[ -f {} ]]", shellescape(path.as_str().unwrap())),
            ast::CmdCond(ref command) => command.to_bash(),
            ast::And(ref cond1, ref cond2) => format!("{{ {} && {}; }}", cond1.to_bash(), cond2.to_bash()),
            ast::Or(ref cond1, ref cond2) => format!("{{ {} || {}; }}", cond1.to_bash(), cond2.to_bash()),
            ast::Not(ref condition) => format!("{{ ! {}; }}", condition.to_bash()),
        }
    }
}

#[cfg(test)]
mod test {
    use super::ToBash;
    use ast;
    use std::path::Path;

    fn cmd() -> ast::Statement {
        ast::Cmd(ast::Raw("hello world".to_string()), vec![])
    }

    #[test]
    fn test_statement_to_bash() {
        assert_eq!("travis_cmd hello\\ world", cmd().to_bash().as_slice());
        assert_eq!("travis_cmd hello\\ world --echo --display=this\\ is\\ output --assert", ast::Cmd(ast::Raw("hello world".to_string()), vec![ast::EchoOption, ast::DisplayOption("this is output".to_string()), ast::AssertOption]).to_bash().as_slice());
        assert_eq!("travis_fold start hello\ntravis_cmd hello\\ world\ntravis_fold end hello", ast::Fold("hello".to_string(), box cmd()).to_bash().as_slice());
        assert_eq!("", ast::Noop.to_bash().as_slice());
        assert_eq!("if true; then\n  travis_cmd hello\\ world\nfi", ast::If(ast::CmdCond(ast::Raw("true".to_string())), box cmd(), box ast::Noop).to_bash().as_slice());
        assert_eq!("travis_cmd hello\\ world\ntravis_cmd hello\\ world", ast::Statements(box vec![cmd(), cmd()]).to_bash().as_slice());
    }

    #[test]
    fn test_command_to_bash() {
        assert_eq!("foo bar", ast::Raw("foo bar".to_string()).to_bash().as_slice());
        assert_eq!("echo foo bar", ast::Echo("foo bar".to_string()).to_bash().as_slice());
        assert_eq!("echo", ast::Newline.to_bash().as_slice());
        assert_eq!("export FOO=bar\\ baz", ast::Envset("FOO".to_string(), "bar baz".to_string()).to_bash().as_slice());
        assert_eq!("cd path/to/some\\ where", ast::Cd(Path::new(b"path/to/some where")).to_bash().as_slice());
        assert_eq!("base64 --decode > path/to/file <<<aGVsbG8gd29ybGQ\\=", ast::Putfile(Path::new("path/to/file"), b"hello world".to_vec()).to_bash().as_slice());
        assert_eq!("mkdir -p path/to/dir", ast::Mkdir(Path::new("path/to/dir")).to_bash().as_slice());
        assert_eq!("cp -r path/from path/to", ast::Copyfile(Path::new("path/from"), Path::new("path/to")).to_bash().as_slice());
        assert_eq!("mv path/from path/to", ast::Movefile(Path::new("path/from"), Path::new("path/to")).to_bash().as_slice());
        assert_eq!("rm -rf path/to/remove", ast::Removefile(Path::new("path/to/remove")).to_bash().as_slice());
    }

    #[test]
    fn test_condition_to_bash() {
        assert_eq!("[[ -e this/is\\ the/path ]]", ast::Exists(Path::new("this/is the/path")).to_bash().as_slice());
        assert_eq!("[[ -d this/is\\ the/path ]]", ast::IsDirectory(Path::new("this/is the/path")).to_bash().as_slice());
        assert_eq!("[[ -f this/is\\ the/path ]]", ast::IsFile(Path::new("this/is the/path")).to_bash().as_slice());
        assert_eq!("hello world", ast::CmdCond(ast::Raw("hello world".to_string())).to_bash().as_slice());
        assert_eq!("{ this && that; }", ast::And(box ast::CmdCond(ast::Raw("this".to_string())), box ast::CmdCond(ast::Raw("that".to_string()))).to_bash().as_slice());
        assert_eq!("{ this || that; }", ast::Or(box ast::CmdCond(ast::Raw("this".to_string())), box ast::CmdCond(ast::Raw("that".to_string()))).to_bash().as_slice());
        assert_eq!("{ ! this; }", ast::Not(box ast::CmdCond(ast::Raw("this".to_string()))).to_bash().as_slice());
    }
}
