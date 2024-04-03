use regex::Regex;
use std::io::Result;
use toml::value::Table;

#[derive(Clone)]
pub struct Rule {
    pub pattern: Regex,
    pub replacement: String,
}

#[derive(Clone)]
pub struct Profile {
    pub program: String,
    pub argmatch: String,
    pub rules: Vec<Rule>,
}

#[derive(Clone)]
pub struct Config {
    pub profiles: Vec<Profile>,
}

impl Config {
    pub fn from_toml(value: Table) -> Result<Config> {
        let mut profiles = Vec::new();
        for (key, value) in value.iter() {
            for (subkey, value) in value.as_table().unwrap().iter() {
                let program = key.to_string();
                let argmatch = subkey.to_string();
                let mut null_rules = Vec::new();
                for rule in value.get("rules").unwrap().as_array().unwrap() {
                    let rule = rule.as_array().unwrap();
                    let pattern = Regex::new(rule[0].as_str().unwrap()).unwrap();
                    let replacement = rule[1].as_str().unwrap().to_string();
                    null_rules.push(Rule {
                        pattern,
                        replacement,
                    });
                }
                profiles.push(Profile {
                    program,
                    argmatch,
                    rules: null_rules,
                });
            }
        }
        Ok(Config { profiles })
    }

    pub fn from_str(cfg: &str) -> Result<Config> {
        let value: Table = cfg.parse().unwrap();
        Config::from_toml(value)
    }

    pub fn from_file(path: &str) -> Result<Config> {
        let contents = std::fs::read_to_string(path)?;
        Config::from_str(&contents)
    }
}

mod tests {
    #[test]
    fn test_config_from_toml() {
        use super::*;

        let toml = r#"
			[gh-copilot."*"]
			rules = [
				["Welcome.*\n", ""],
				["version.*\n", ""],
				["I'm powered.*\n", ""],
				["\n\n", "\n"],
			]
			
			[kubectl." "]
			rules = [
				["Find more.*\n", ""],
			]

			[kubectl."version"]
			rules = [
				["\n\n", "\n"],
			]
		"#;
        let value: Table = toml::from_str(toml).unwrap();
        let conf = Config::from_toml(value).unwrap();
        let want: Config = Config {
            profiles: vec![
                Profile {
                    program: "gh-copilot".to_string(),
                    argmatch: "*".to_string(),
                    rules: vec![
                        Rule {
                            pattern: Regex::new("Welcome.*\n").unwrap(),
                            replacement: "".to_string(),
                        },
                        Rule {
                            pattern: Regex::new("version.*\n").unwrap(),
                            replacement: "".to_string(),
                        },
                        Rule {
                            pattern: Regex::new("I'm powered.*\n").unwrap(),
                            replacement: "".to_string(),
                        },
                        Rule {
                            pattern: Regex::new("\n\n").unwrap(),
                            replacement: "\n".to_string(),
                        },
                    ],
                },
                Profile {
                    program: "kubectl".to_string(),
                    argmatch: " ".to_string(),
                    rules: vec![Rule {
                        pattern: Regex::new("Find more.*\n").unwrap(),
                        replacement: "".to_string(),
                    }],
                },
                Profile {
                    program: "kubectl".to_string(),
                    argmatch: "version".to_string(),
                    rules: vec![Rule {
                        pattern: Regex::new("\n\n").unwrap(),
                        replacement: "\n".to_string(),
                    }],
                },
            ],
        };
        assert_eq!(conf.profiles.len(), want.profiles.len());
        for (i, profile) in conf.profiles.iter().enumerate() {
            assert_eq!(profile.program, want.profiles[i].program);
            assert_eq!(profile.argmatch, want.profiles[i].argmatch);
            assert_eq!(profile.rules.len(), want.profiles[i].rules.len());
            for (j, rule) in profile.rules.iter().enumerate() {
                assert_eq!(
                    rule.pattern.as_str(),
                    want.profiles[i].rules[j].pattern.as_str()
                );
                assert_eq!(rule.replacement, want.profiles[i].rules[j].replacement);
            }
        }
    }
}
