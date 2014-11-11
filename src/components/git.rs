use ast;
use payload::Payload;
use bash;

pub fn git_checkout_ast(payload: &Payload) -> ast::Statement {
    ast_block!(
        cmd!(ast::Mkdir(Path::new("/home/travis/build")));
        cmd!(ast::Cd(Path::new("build")));
        ast_set!(GIT_ASKPASS = "echo".to_string());

        ast_if! (!ast::IsDirectory(git_path(payload).join(Path::new(".git"))) {
            format_cmd!([EchoOption|AssertOption], "git clone {} {} {}", git_clone_args(payload), git_source_url(payload), git_path(payload).as_str().unwrap());
        } else {
            format_cmd!([EchoOption|AssertOption], "git -C {} fetch origin", git_path(payload).as_str().unwrap());
            format_cmd!([EchoOption|AssertOption], "git -C {} reset --hard", git_path(payload).as_str().unwrap());
        });

        cmd!(ast::Cd(git_path(payload)));
        match payload.job.git_ref {
            Some(ref git_ref) => format_cmd!([EchoOption|AssertOption], "git fetch origin +{}:", git_ref),
            None => ast::Noop,
        };

        format_cmd!([EchoOption|AssertOption], "git checkout -qf {}", if payload.job.pull_request { "FETCH_HEAD" } else { payload.job.commit.as_slice() });

        if payload.config.git.submodules {
            ast_if! (ast::IsFile(Path::new(".gitmodules")) {
                format_cmd!([EchoOption], "git submodule init");
                format_cmd!([EchoOption], "git submodule update{}", submodules_args(payload));
            })
        } else {
            ast::Noop
        };
    )
}

fn git_path(payload: &Payload) -> Path {
    Path::new(payload.repository.slug.as_slice())
}

fn git_clone_args(payload: &Payload) -> String {
    if payload.job.git_ref.is_some() {
        format!("--depth={}", payload.config.git.depth)
    } else {
        format!("--depth={} --branch={}", payload.config.git.depth, bash::shellescape(payload.job.branch.as_slice()))
    }
}

fn submodules_args(payload: &Payload) -> String {
    match payload.config.git.submodules_depth {
        Some(depth) => format!(" --depth={}", depth),
        None => "".to_string(),
    }
}

fn git_source_url(payload: &Payload) -> &str {
    payload.repository.source_url.as_slice()
}

#[cfg(test)]
mod test {
    use super::git_checkout_ast;
    use test_ast_runner::TestAstRunner;
    use payload::test::a_payload;

    fn assert_command_was_run(runner: &TestAstRunner, expected: &str) {
        assert!(runner.commands.iter().any(|&(ref command, _)| command.as_slice() == expected), "expected command '{}' to be run, but wasn't: {}, (files: {})", expected, runner.commands, runner.fs_state);
    }

    fn assert_command_was_not_run_start(runner: &TestAstRunner, expected: &str) {
        assert!(!runner.commands.iter().any(|&(ref command, _)| command.as_slice().starts_with(expected)), "expected command '{}' to not be run, but was: {} (files: {})", expected, runner.commands, runner.fs_state);
    }

    #[test]
    fn test_cd_to_clone() {
        let payload = a_payload();
        let script = git_checkout_ast(&payload);
        let mut runner = TestAstRunner::new();
        runner.run(&script);

        assert_eq!(Some("/home/travis/build/example_owner/example_repo"), runner.working_directory.as_str());
    }

    #[test]
    fn test_git_clone() {
        let payload = a_payload();
        let script = git_checkout_ast(&payload);
        let mut runner = TestAstRunner::new();
        runner.run(&script);

        assert_command_was_run(&runner, "git clone --depth=50 --branch=master git://github.com/example_owner/example_repo.git example_owner/example_repo");
    }

    #[test]
    fn test_git_clone_custom_depth() {
        let mut payload = a_payload();
        payload.config.git.depth = 1;

        let script = git_checkout_ast(&payload);
        let mut runner = TestAstRunner::new();
        runner.run(&script);

        assert_command_was_run(&runner, "git clone --depth=1 --branch=master git://github.com/example_owner/example_repo.git example_owner/example_repo");
    }

    #[test]
    fn test_git_clone_escape_branch() {
        let mut payload = a_payload();
        payload.job.branch = "a->b".to_string();

        let script = git_checkout_ast(&payload);
        let mut runner = TestAstRunner::new();
        runner.run(&script);

        assert_command_was_run(&runner, "git clone --depth=50 --branch=a-\\>b git://github.com/example_owner/example_repo.git example_owner/example_repo");
    }

    #[test]
    fn test_does_not_fetch_ref() {
        let payload = a_payload();
        let script = git_checkout_ast(&payload);
        let mut runner = TestAstRunner::new();
        runner.run(&script);

        assert_command_was_not_run_start(&runner, "git fetch");
    }

    #[test]
    fn test_fetch_ref() {
        let mut payload = a_payload();
        payload.job.git_ref = Some("refs/pull/118/merge".to_string());

        let script = git_checkout_ast(&payload);
        let mut runner = TestAstRunner::new();
        runner.run(&script);

        assert_command_was_run(&runner, "git fetch origin +refs/pull/118/merge:");
    }

    #[test]
    fn test_check_out_commit() {
        let payload = a_payload();
        let script = git_checkout_ast(&payload);
        let mut runner = TestAstRunner::new();
        runner.run(&script);

        assert_command_was_run(&runner, "git checkout -qf abcdef");
    }

    #[test]
    fn test_check_out_pull_request() {
        let mut payload = a_payload();
        payload.job.git_ref = Some("refs/pull/118/merge".to_string());
        payload.job.pull_request = true;

        let script = git_checkout_ast(&payload);
        let mut runner = TestAstRunner::new();
        runner.run(&script);

        assert_command_was_run(&runner, "git checkout -qf FETCH_HEAD");
    }

    #[test]
    fn test_submodules() {
        let payload = a_payload();
        let script = git_checkout_ast(&payload);
        let mut runner = TestAstRunner::new();
        runner.mkdir(&Path::new("/home/travis/build"));
        runner.mkdir(&Path::new("/home/travis/build/example_owner"));
        runner.mkdir(&Path::new("/home/travis/build/example_owner/example_repo"));
        runner.put_file(&Path::new("/home/travis/build/example_owner/example_repo/.gitmodules"), b"hello there");
        runner.run(&script);

        assert_command_was_run(&runner, "git submodule init");
        assert_command_was_run(&runner, "git submodule update");
    }

    #[test]
    fn test_submodules_custom_depth() {
        let mut payload = a_payload();
        payload.config.git.submodules_depth = Some(10);

        let script = git_checkout_ast(&payload);
        let mut runner = TestAstRunner::new();
        runner.mkdir(&Path::new("/home/travis/build"));
        runner.mkdir(&Path::new("/home/travis/build/example_owner"));
        runner.mkdir(&Path::new("/home/travis/build/example_owner/example_repo"));
        runner.put_file(&Path::new("/home/travis/build/example_owner/example_repo/.gitmodules"), b"hello there");
        runner.run(&script);

        assert_command_was_run(&runner, "git submodule update --depth=10");
    }

    #[test]
    fn test_submodules_disabled() {
        let mut payload = a_payload();
        payload.config.git.submodules = false;

        let script = git_checkout_ast(&payload);
        let mut runner = TestAstRunner::new();
        runner.mkdir(&Path::new("/home/travis/build"));
        runner.mkdir(&Path::new("/home/travis/build/example_owner"));
        runner.mkdir(&Path::new("/home/travis/build/example_owner/example_repo"));
        runner.put_file(&Path::new("/home/travis/build/example_owner/example_repo/.gitmodules"), b"hello there");
        runner.run(&script);

        assert_command_was_not_run_start(&runner, "git submodule");
    }
}
