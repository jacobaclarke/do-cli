use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::marker::PhantomData;
use std::path::PathBuf;

use serde::de;
use serde::de::Deserializer;

fn string_or_seq_string<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    struct StringOrVec(PhantomData<Vec<String>>);

    impl<'de> de::Visitor<'de> for StringOrVec {
        type Value = Vec<String>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string or list of strings")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(vec![value.to_owned()])
        }

        fn visit_seq<S>(self, visitor: S) -> Result<Self::Value, S::Error>
        where
            S: de::SeqAccess<'de>,
        {
            Deserialize::deserialize(de::value::SeqAccessDeserializer::new(visitor))
        }
    }

    deserializer.deserialize_any(StringOrVec(PhantomData))
}

#[derive(Deserialize, Serialize, Debug, Default, PartialEq, Eq)]
pub struct Conf {
    #[serde(default)]
    pub tasks: IndexMap<String, Task>,
    #[serde(default)]
    pub env: IndexMap<String, String>,
}

#[derive(Deserialize, Debug, PartialEq, Eq, Serialize, Default)]
pub struct Task {
    #[serde(deserialize_with = "string_or_seq_string")]
    pub cmd: Vec<String>,
    #[serde(default)]
    pub env: IndexMap<String, String>,
    #[serde(default)]
    pub workdir: Option<PathBuf>,
    #[serde(default)]
    pub local: bool,
    #[serde(default)]
    pub hidden: bool,
    #[serde(default)]
    pub display: bool,
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
    fn test_deserialize_cmds_handling() {
        let conf = serde_yaml::from_str::<Conf>(
            r#"
            tasks:
                booba:
                    cmd : ["hi"]
        "#,
        ).unwrap();
        assert_eq!(conf.tasks["booba"].cmd, vec!["hi"]);
    }

    #[test]
    fn test_task_default_visible() {
        assert!(!Task::default().hidden);
    }

    #[test]
    fn test_deserialize_cmds() -> anyhow::Result<()> {
        let text = r#"
tasks: 
    hello:
        cmd: echo hello"#;
        let conf: Conf = serde_yaml::from_str(text)?;
        assert_eq!(conf.tasks["hello"].cmd[0], "echo hello");
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
        let mut conf: Conf = serde_yaml::from_str(parent)?;

        let child_conf: Conf = serde_yaml::from_str(child)?;

        conf.extend(child_conf);
        assert_eq!(conf.tasks["hello"].cmd[0], r#"echo hello $NAME"#);
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
        let conf: Conf = serde_yaml::from_str(text)?;
        assert!(!conf.tasks["hello"].local);
        Ok(())
    }
}
