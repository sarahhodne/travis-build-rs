use serialize::json;

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

impl Payload {
    pub fn from_json(j: &json::Json) -> json::DecodeResult<Payload> {
        if !j.is_object() {
            return Err(json::ApplicationError("payload must be an object".to_string()));
        }

        let job = match j.find(&"job".to_string()) {
            Some(j) => try!(Job::from_json(j)),
            None => return Err(json::MissingFieldError("job".to_string())),
        };
        let repository = match j.find(&"repository".to_string()) {
            Some(r) => try!(Repository::from_json(r)),
            None => return Err(json::MissingFieldError("repository".to_string())),
        };
        let config = match j.find(&"config".to_string()) {
            Some(c) => try!(Config::from_json(c)),
            None => return Err(json::MissingFieldError("config".to_string())),
        };
        let paranoid = try!(j.find(&"paranoid".to_string()).unwrap_or(&json::Boolean(false)).as_boolean().ok_or(json::ApplicationError("paranoid must be a bool".to_string())));
        let fix_resolv_conf = !try!(j.find(&"skip_resolv_updates".to_string()).unwrap_or(&json::Boolean(true)).as_boolean().ok_or(json::ApplicationError("skip_resolv_updates must be a bool".to_string())));
        let fix_etc_hosts = !try!(j.find(&"skip_etc_hosts_fix".to_string()).unwrap_or(&json::Boolean(true)).as_boolean().ok_or(json::ApplicationError("skip_etc_hosts_fix must be a bool".to_string())));

        Ok(Payload {
            job: job,
            repository: repository,
            config: config,
            paranoid: paranoid,
            fix_resolv_conf: fix_resolv_conf,
            fix_etc_hosts: fix_etc_hosts,
        })
    }
}

impl Job {
    pub fn from_json(j: &json::Json) -> json::DecodeResult<Job> {
        if !j.is_object() {
            return Err(json::ApplicationError("job must be an object".to_string()));
        }

        let branch = match j.find(&"branch".to_string()) {
            Some(u) => try!(u.as_string().ok_or(json::ApplicationError("job.branch must be a string".to_string()))),
            None => return Err(json::MissingFieldError("branch".to_string())),
        };
        let commit = match j.find(&"commit".to_string()) {
            Some(u) => try!(u.as_string().ok_or(json::ApplicationError("job.commit must be a string".to_string()))),
            None => return Err(json::MissingFieldError("commit".to_string())),
        };
        let git_ref = j.find(&"ref".to_string()).map(|r| r.as_string().ok_or(json::ApplicationError("job.ref must be a string".to_string()))).map(|r| r.to_string());
        let pull_request = match j.find(&"pull_request".to_string()) {
            Some(u) => try!(u.as_boolean().ok_or(json::ApplicationError("job.pull_request must be a boolean".to_string()))),
            None => return Err(json::MissingFieldError("pull_request".to_string())),
        };

        Ok(Job {
            branch: branch.to_string(),
            commit: commit.to_string(),
            git_ref: git_ref,
            pull_request: pull_request,
        })
    }
}

impl Repository {
    pub fn from_json(j: &json::Json) -> json::DecodeResult<Repository> {
        if !j.is_object() {
            return Err(json::ApplicationError("repository must be an object".to_string()));
        }

        let slug = match j.find(&"slug".to_string()) {
            Some(u) => try!(u.as_string().ok_or(json::ApplicationError("repository.branch must be a string".to_string()))),
            None => return Err(json::MissingFieldError("slug".to_string())),
        };
        let source_url = match j.find(&"source_url".to_string()) {
            Some(u) => try!(u.as_string().ok_or(json::ApplicationError("repository.commit must be a string".to_string()))),
            None => return Err(json::MissingFieldError("source_url".to_string())),
        };

        Ok(Repository {
            slug: slug.to_string(),
            source_url: source_url.to_string(),
        })
    }
}

impl Config {
    pub fn from_json(j: &json::Json) -> json::DecodeResult<Config> {
        let git_config = try!(GitConfig::from_json(j.find(&"git".to_string()).unwrap_or(&json::Null)));
        let services_json: Vec<json::Json> = try!(j.find(&"services".to_string()).unwrap_or(&json::List(vec![])).as_list().ok_or(json::ApplicationError("config.services must be a list of strings".to_string()))).to_vec();
        let services = services_json.map_in_place(|e| e.as_string().unwrap().to_string());

        Ok(Config {
            git: git_config,
            services: services,
        })
    }
}

impl GitConfig {
    pub fn from_json(j: &json::Json) -> json::DecodeResult<GitConfig> {
        let depth = try!(j.find(&"depth".to_string()).unwrap_or(&json::U64(50)).as_u64().ok_or(json::ApplicationError("git.depth must be an int".to_string())));
        let submodules = try!(j.find(&"submodules".to_string()).unwrap_or(&json::Boolean(true)).as_boolean().ok_or(json::ApplicationError("git.submodules must be a bool".to_string())));
        let submodules_depth = try!(j.find(&"submodules_depth".to_string()).map(|d| d.as_u64()).ok_or(json::ApplicationError("git.submodules_depth must be an int".to_string())));
        let strategy = try!(GitStrategy::from_json(j.find(&"strategy".to_string()).unwrap_or(&json::String("clone".to_string()))));

        Ok(GitConfig {
            depth: depth,
            submodules: submodules,
            submodules_depth: submodules_depth,
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
