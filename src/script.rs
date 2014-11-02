use payload::Payload;
use ast;
use bash::ToBash;
use components;
use script_templates::{SCRIPT_HEADER,SCRIPT_FOOTER};

pub struct Script {
    payload: Payload,
}

impl Script {
    pub fn new(payload: Payload) -> Script {
        Script { payload: payload }
    }

    pub fn to_script(&self) -> String {
        let ast = self.generate_ast();

        let mut script = SCRIPT_HEADER.to_string();
        script.push_str(ast.to_bash().as_slice());
        script.push('\n');
        script.push_str(SCRIPT_FOOTER);

        script
    }

    fn generate_ast(&self) -> ast::Statement {
        ast_block! {
            self.builtin_stages_ast();
            self.custom_stages_ast();
        }
    }

    fn builtin_stages_ast(&self) -> ast::Statement {
        ast_block! {
            self.apply_fixes();
            components::git::git_checkout_ast(&self.payload);
            components::services::start_services_ast(&self.payload);
        }
    }

    fn custom_stages_ast(&self) -> ast::Statement {
        ast::Noop
    }

    fn apply_fixes(&self) -> ast::Statement {
        ast_block! {
            if self.payload.fix_resolv_conf {
                format_cmd!("grep '199.91.168' /etc/resolv.conf > /dev/null || echo 'nameserver 199.91.168.70\nnameserver 199.91.168.71' | sudo tee /etc/resolv.conf &> /dev/null")
            } else {
                ast::Noop
            };
            if self.payload.fix_etc_hosts {
                format_cmd!("sudo sed -e 's/^\\(127\\.0\\.0\\.1.*\\)$/\\1 '`hostname`'/' -i'.bak' /etc/hosts")
            } else {
                ast::Noop
            };
        }
    }
}
