#![cfg(test)]

use ast;
use std::collections::HashMap;

pub type Directory = HashMap<String, DirectoryEntry>;

#[deriving(Show)]
pub enum DirectoryEntry {
    Dir(HashMap<String, DirectoryEntry>),
    File(Vec<u8>)
}

pub struct TestAstRunner {
    pub fs_state: DirectoryEntry,
    pub working_directory: Path,
    pub commands: Vec<(String, Vec<ast::CommandOption>)>,
    pub environment_vars: HashMap<String, String>,
}

impl DirectoryEntry {
    pub fn walk_mut<'a>(&'a mut self, path: &[&str]) -> Option<&'a mut DirectoryEntry> {
        if path.len() == 0 {
            return Some(self)
        }

        match *self {
            Dir(ref mut dir) => dir.find_mut(&path.head().unwrap().to_string()).and_then(|d| d.walk_mut(path.tail())),
            File(_) => None,
        }
    }

    pub fn walk<'a>(&'a self, path: &[&str]) -> Option<&'a DirectoryEntry> {
        if path.len() == 0 {
            return Some(self)
        }

        match *self {
            Dir(ref dir) => dir.find(&path.head().unwrap().to_string()).and_then(|d| d.walk(path.tail())),
            File(_) => None,
        }
    }
}

impl TestAstRunner {
    pub fn new() -> TestAstRunner {
        let fs: HashMap<String, DirectoryEntry> = HashMap::new();

        let mut runner = TestAstRunner {
            fs_state: Dir(fs),
            commands: Vec::new(),
            working_directory: Path::new("/home/travis"),
            environment_vars: HashMap::new(),
        };

        runner.mkdir(&Path::new("/home"));
        runner.mkdir(&Path::new("/home/travis"));

        runner
    }

    pub fn run(&mut self, script: &ast::Statement) {
        self.run_statement(script);
    }

    pub fn mkdir(&mut self, path: &Path) {
        let dir_path = self.working_directory.join(path).dir_path();
        let path_parts: Vec<&str> = dir_path.str_components().map(|c| c.unwrap()).collect();
        let parent_dir = self.fs_state.walk_mut(path_parts.as_slice());
        match parent_dir {
            Some(entry) => match *entry {
                Dir(ref mut d) => {
                    if !d.contains_key(&path.filename_str().unwrap().to_string()) {
                        d.insert(path.filename_str().unwrap().to_string(), Dir(HashMap::new()));
                    };
                }
                File(_) => panic!("attempting to mkdir in a subdir of a file"),
            },
            None => panic!("attempting to mkdir in a subdir of non-existant"),
        };
    }

    pub fn put_file(&mut self, path: &Path, body: &[u8]) {
        let dir_path = self.working_directory.join(path).dir_path();
        let path_parts: Vec<&str> = dir_path.str_components().map(|c| c.unwrap()).collect();
        let parent_dir = self.fs_state.walk_mut(path_parts.as_slice());
        match parent_dir {
            Some(entry) => match *entry {
                Dir(ref mut d) => d.insert(path.filename_str().unwrap().to_string(), File(body.to_vec())),
                File(_) => panic!("attempting to write file to a subdir of a file"),
            },
            None => panic!("attempting to write a file to a subdir of non-existant"),
        };
    }

    fn run_statement(&mut self, statement: &ast::Statement) {
        match *statement {
            ast::Statements(ref stmts) => {
                for stmt in stmts.iter() {
                    self.run_statement(stmt);
                }
            },
            ast::Fold(_, box ref stmt) => self.run_statement(stmt),
            ast::Cmd(ref cmd, ref opts) => self.run_command(cmd, opts),
            ast::If(ref cond, box ref thenbody, box ref elsebody) => self.run_if(cond, thenbody, elsebody),
            ast::Noop => {}
        }
    }

    fn run_command(&mut self, command: &ast::Command, opts: &Vec<ast::CommandOption>) {
        match *command {
            ast::Raw(ref cmd) => self.commands.push((cmd.clone(), opts.clone())),
            ast::Echo(ref text) => self.commands.push((format!("echo {}", text), opts.clone())),
            ast::Newline => self.commands.push(("echo".to_string(), opts.clone())),
            ast::Envset(ref key, ref value) => {
                self.environment_vars.insert(key.clone(), value.clone());
            },
            // TODO: Check if path exists
            ast::Cd(ref path) => self.working_directory = self.working_directory.join(path),
            ast::Putfile(ref path, ref body) => self.put_file(path, body.as_slice()),
            ast::Mkdir(ref path) => self.mkdir(path),
            _ => unimplemented!(),
        }
    }

    fn run_if(&mut self, condition: &ast::Condition, thenbody: &ast::Statement, elsebody: &ast::Statement) {
        if self.eval_condition(condition) {
            self.run_statement(thenbody);
        } else {
            self.run_statement(elsebody);
        }
    }

    fn eval_condition(&mut self, condition: &ast::Condition) -> bool {
        match *condition {
            ast::Exists(ref path) => {
                let full_path = self.working_directory.join(path);
                let path_parts: Vec<&str> = full_path.str_components().map(|c| c.unwrap()).collect();
                self.fs_state.walk(path_parts.as_slice()).is_some()
            },
            ast::IsDirectory(ref path) => {
                let full_path = self.working_directory.join(path);
                let path_parts: Vec<&str> = full_path.str_components().map(|c| c.unwrap()).collect();
                match self.fs_state.walk(path_parts.as_slice()) {
                    Some(&Dir(_)) => true,
                    _ => false,
                }
            },
            ast::IsFile(ref path) => {
                let full_path = self.working_directory.join(path);
                let path_parts: Vec<&str> = full_path.str_components().map(|c| c.unwrap()).collect();
                match self.fs_state.walk(path_parts.as_slice()) {
                    Some(&File(_)) => true,
                    _ => false,
                }
            },
            ast::CmdCond(_) => unimplemented!(),
            ast::And(box ref cond1, box ref cond2) => self.eval_condition(cond1) && self.eval_condition(cond2),
            ast::Or(box ref cond1, box ref cond2) => self.eval_condition(cond1) || self.eval_condition(cond2),
            ast::Not(box ref cond) => !self.eval_condition(cond),
        }
    }
}
