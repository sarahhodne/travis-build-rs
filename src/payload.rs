use serialize::json;
use std::string::as_string;

pub struct Payload {
    pub job: Job,
    pub repository: Repository,
    pub config: Config,
    pub paranoid: bool,
    pub fix_resolv_conf: bool,
    pub fix_etc_hosts: bool,
}

pub struct Job {
    pub branch: String,
    pub commit: String,
    pub git_ref: Option<String>,
    pub pull_request: bool,
}

pub struct Repository {
    pub slug: String,
    pub source_url: String,
}

pub struct Config {
    pub language: String,
    pub git: GitConfig,
    pub services: Vec<String>,
}

pub struct GitConfig {
    pub depth: u64,
    pub submodules: bool,
    pub submodules_depth: Option<u64>,
    pub strategy: GitStrategy,
}

pub enum GitStrategy {
    Clone,
    Tarball,
}

macro_rules! json_to (
    ($r:expr, String) => ($r.as_string());
    ($r:expr, bool) => ($r.as_boolean());
    ($r:expr, u64) => ($r.as_u64());
    ($r:expr, $t:ident) => (Some(try!($t::from_json($r))));
)

macro_rules! find_key (
        ($j:expr, $t:ident, $key:expr) => (match $j.find(&*::std::string::as_string($key)) {
            Some(v) => try!(json_to!(v, $t).ok_or(json::ApplicationError(format!("{} must be a string", $key)))),
            None => return Err(json::MissingFieldError($key.to_string())),
        });
        ($j:expr, $t:ident, $key:expr, Optional) => (try!($j.find(&*::std::string::as_string($key)).map_or(Ok(None), |r| json_to!(r, $t).ok_or(json::ApplicationError(format!("{} must be a string", $key))).map(Some))));
        ($j:expr, $t:ident, $key:expr, $default:expr) => (match $j.find(&*::std::string::as_string($key)) {
            Some(v) => try!(json_to!(v, $t).ok_or(json::ApplicationError(format!("{} must be a string", $key)))),
            None => $default,
        });
)

impl Payload {
    pub fn from_json(j: &json::Json) -> json::DecodeResult<Payload> {
        if !j.is_object() {
            return Err(json::ApplicationError("payload must be an object".to_string()));
        }

        Ok(Payload {
            job: find_key!(j, Job, "job"),
            repository: find_key!(j, Repository, "repository"),
            config: find_key!(j, Config, "repository"),
            paranoid: find_key!(j, bool, "paranoid", false),
            fix_resolv_conf: !find_key!(j, bool, "skip_resolv_updates", true),
            fix_etc_hosts: !find_key!(j, bool, "skip_etc_hosts_fix", true),
        })
    }
}

impl Job {
    pub fn from_json(j: &json::Json) -> json::DecodeResult<Job> {
        if !j.is_object() {
            return Err(json::ApplicationError("job must be an object".to_string()));
        }

        Ok(Job {
            branch: find_key!(j, String, "branch").to_string(),
            commit: find_key!(j, String, "commit").to_string(),
            git_ref: find_key!(j, String, "ref", Optional).map(|s| s.to_string()),
            pull_request: find_key!(j, bool, "pull_request"),
        })
    }
}

impl Repository {
    pub fn from_json(j: &json::Json) -> json::DecodeResult<Repository> {
        if !j.is_object() {
            return Err(json::ApplicationError("repository must be an object".to_string()));
        }

        Ok(Repository {
            slug: find_key!(j, String, "slug").to_string(),
            source_url: find_key!(j, String, "source_url").to_string(),
        })
    }
}

impl Config {
    pub fn from_json(j: &json::Json) -> json::DecodeResult<Config> {
        // let git_config = try!(GitConfig::from_json(j.find(&"git".to_string()).unwrap_or(&json::Null)));
        let services_json: Vec<json::Json> = try!(j.find(&"services".to_string()).unwrap_or(&json::List(vec![])).as_list().ok_or(json::ApplicationError("config.services must be a list of strings".to_string()))).to_vec();
        let services = services_json.iter().map(|e| e.as_string().unwrap().to_string()).collect();

        Ok(Config {
            language: find_key!(j, String, "language", "ruby").to_string(),
            git: find_key!(j, GitConfig, "git", GitConfig::default()),
            services: services,
        })
    }
}

impl GitConfig {
    pub fn default() -> GitConfig {
        GitConfig {
            depth: 50,
            submodules: true,
            submodules_depth: None,
            strategy: Clone,
        }
    }

    pub fn from_json(j: &json::Json) -> json::DecodeResult<GitConfig> {
        let strategy = try!(GitStrategy::from_json(j.find(&*as_string("strategy")).unwrap_or(&json::String("clone".to_string()))));

        Ok(GitConfig {
            depth: find_key!(j, u64, "depth", 50),
            submodules: find_key!(j, bool, "submodules", true),
            submodules_depth: find_key!(j, u64, "submodules_depth", Optional),
            strategy: strategy,
        })
    }
}

impl GitStrategy {
    pub fn from_json(j: &json::Json) -> json::DecodeResult<GitStrategy> {
        match j.as_string().unwrap_or("clone") {
            "tarball" => Ok(Tarball),
            "clone" => Ok(Clone),
            _ => Err(json::ApplicationError("git.strategy unknown".to_string())),
        }
    }
}

#[cfg(test)]
pub mod test {
    use super::{Payload,Job,Repository,Config,GitConfig,Clone};

    pub fn a_payload() -> Payload {
        Payload {
            job: Job {
                branch: "master".to_string(),
                commit: "abcdef".to_string(),
                git_ref: None,
                pull_request: false,
            },
            repository: Repository {
                slug: "example_owner/example_repo".to_string(),
                source_url: "git://github.com/example_owner/example_repo.git".to_string(),
            },
            config: Config {
                git: GitConfig {
                    depth: 50,
                    submodules: true,
                    submodules_depth: None,
                    strategy: Clone,
                },
                services: vec![],
            },
            paranoid: false,
            fix_resolv_conf: false,
            fix_etc_hosts: false,
        }
    }
}
