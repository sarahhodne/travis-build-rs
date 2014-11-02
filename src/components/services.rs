use ast;
use payload::Payload;

pub fn start_services_ast(payload: &Payload) -> ast::Statement {
    let stmts = payload.config.services.clone()
        .map_in_place(normalize_service)
        .iter()
        .map(|service| format_cmd!([EchoOption], "sudo service {} start", service))
        .collect();

    ast::Statements(box stmts)
}

fn normalize_service(service: String) -> String {
    match service.as_slice() {
        "hbase" => "hbase-master".to_string(),
        "memcache" => "memcached".to_string(),
        "neo4j-server" => "neo4j".to_string(),
        "rabbitmq" => "rabbitmq-server".to_string(),
        "redis" => "redis-server".to_string(),
        _ => service,
    }
}

#[cfg(test)]
mod test {
    use super::start_services_ast;
    use test_ast_runner::TestAstRunner;
    use payload::test::a_payload;

    fn assert_command_was_run(runner: &TestAstRunner, expected: &str) {
        assert!(runner.commands.iter().any(|&(ref command, _)| command.as_slice() == expected), "expected command '{}' to be run, but wasn't: {}, (files: {})", expected, runner.commands, runner.fs_state);
    }

    fn assert_command_was_not_run_start(runner: &TestAstRunner, expected: &str) {
        assert!(!runner.commands.iter().any(|&(ref command, _)| command.as_slice().starts_with(expected)), "expected command '{}' to not be run, but was: {} (files: {})", expected, runner.commands, runner.fs_state);
    }

    #[test]
    fn test_no_services() {
        let payload = a_payload();
        let script = start_services_ast(&payload);
        let mut runner = TestAstRunner::new();
        runner.run(&script);

        assert_command_was_not_run_start(&runner, "sudo service");
    }

    #[test]
    fn test_service_normalized() {
        let mut payload = a_payload();
        payload.config.services = vec!["redis".to_string()];

        let script = start_services_ast(&payload);
        let mut runner = TestAstRunner::new();
        runner.run(&script);

        assert_command_was_run(&runner, "sudo service redis-server start");
    }

    #[test]
    fn test_service_not_normalized() {
        let mut payload = a_payload();
        payload.config.services = vec!["elasticsearch".to_string()];

        let script = start_services_ast(&payload);
        let mut runner = TestAstRunner::new();
        runner.run(&script);

        assert_command_was_run(&runner, "sudo service elasticsearch start");
    }
}
