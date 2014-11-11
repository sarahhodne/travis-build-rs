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
            self.enable_paranoid_mode();
            self.export_vars();
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

    fn enable_paranoid_mode(&self) -> ast::Statement {
        if !self.payload.paranoid {
            return ast::Noop;
        }

        ast_block! {
            cmd!(ast::Newline);
            cmd!(ast::Echo("Sudo, the Firefox addon, setuid and setgid have been disabled.".to_string()));
            format_cmd!("sudo -n sh -c \"sed -e \\'s/^%.*//\\' -i.bak /etc/sudoers && rm -f /etc/sudoers.d/travis && find / -perm -4000 -exec chmod a-s {{}} \\; 2>/dev/null\"");
        }
    }

    fn export_vars(&self) -> ast::Statement {
        ast_block! {
            ast_set!(TRAVIS = "true".to_string());
            ast_set!(CI = "true".to_string());
            ast_set!(CONTINUOUS_INTEGRATION = "true".to_string());
            ast_set!(HAS_JOSH_K_SEAL_OF_APPROVAL = "true".to_string());
        }
    }
}
