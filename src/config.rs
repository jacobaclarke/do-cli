use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Deserialize, Serialize, Debug, Default, PartialEq, Eq)]
pub struct Conf {
    #[serde(default)]
    pub tasks: IndexMap<String, Task>,
    #[serde(default)]
    pub env: IndexMap<String, String>,
}

#[derive(Deserialize, Debug, PartialEq, Eq, Serialize)]
pub struct Task {
    pub cmd: String,
    #[serde(default)]
    pub env: IndexMap<String, String>,
    #[serde(default)]
    pub workdir: Option<PathBuf>,
    #[serde(default)]
    pub local: bool,
}

impl Conf {
    pub fn extend(&mut self, other: Conf) {
        self.tasks.extend(other.tasks);
        let mut env = self.env.clone();
        env.extend(other.env);
        self.env = env;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_cmds() -> anyhow::Result<()> {
        let text = r#"
tasks: 
    hello:
        cmd: echo hello"#;
        let conf: Conf = serde_yaml::from_str(&text)?;
        assert_eq!(conf.tasks["hello"].cmd, "echo hello");
        Ok(())
    }

    #[test]
    fn test_default_values() {
        assert_eq!(
            Conf::default(),
            Conf {
                tasks: IndexMap::new(),
                env: IndexMap::new(),
            }
        );
    }

    #[test]
    fn test_defaults() -> anyhow::Result<()> {
        serde_yaml::from_str::<Conf>("")?;
        Ok(())
    }

    #[test]
    fn test_extend() -> anyhow::Result<()> {
        let parent = r#"env:
    NAME: world
"#;
        let child = r#"tasks:
    hello:
        cmd: echo hello $NAME"#;
        let mut conf: Conf = serde_yaml::from_str(&parent)?;

        let child_conf: Conf = serde_yaml::from_str(&child)?;

        conf.extend(child_conf);
        assert_eq!(conf.tasks["hello"].cmd, r#"echo hello $NAME"#);
        Ok(())
    }

    #[test]
    fn test_combine_tasks() -> anyhow::Result<()> {
        let mut conf = serde_yaml::from_str::<Conf>(
            r#"tasks:
    hello:
        cmd: echo hello"#,
        )?;
        let child = serde_yaml::from_str::<Conf>(
            r#"tasks:
    bye:
        cmd: echo bye"#,
        )?;
        conf.extend(child);
        assert_eq!(conf.tasks.len(), 2);
        Ok(())
    }

    #[test]
    fn test_local_default() -> anyhow::Result<()> {
        let text = r#"
tasks:
    hello:
        cmd: echo hello
"#;
        let conf: Conf = serde_yaml::from_str(&text)?;
        assert_eq!(conf.tasks["hello"].local, false);
        Ok(())
    }
}
