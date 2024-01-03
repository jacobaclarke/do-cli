use colored::*;
use indexmap::IndexMap;
use serde::Deserialize;
use std::env;
use std::fs;
use std::io::Write;
use std::iter::Extend;
use std::process::Command;
use std::str::FromStr;
use std::{path::PathBuf, str};
use strum_macros::EnumString;
use tempfile::NamedTempFile;

#[derive(Deserialize, Debug, Default)]
pub struct Conf {
    #[serde(default)]
    tasks: IndexMap<String, Task>,
    #[serde(default)]
    env: IndexMap<String, String>,
}

#[derive(Deserialize, Debug, EnumString, Default, PartialEq)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
enum WorkDir {
    Local,
    #[default]
    None,
    Path(PathBuf),
}

impl Conf {
    pub fn list_commands(&self) {
        println!("Environment:\n--------------------");
        for (key, value) in &self.env {
            println!("{}={}", key.red(), value.blue());
        }
        println!("\nAvailable commands:\n--------------------");
        for (name, _) in &self.tasks {
            println!("{}", name.green());
            for (key, value) in &self.tasks[name].env {
                println!("  {}={}", key.red(), value.blue());
            }
            for row in self.tasks[name].cmd.lines() {
                println!("  {}", row.blue());
            }
        }
    }

    pub fn exec(&self, cmd: Vec<&str>) -> anyhow::Result<String> {
        let name = cmd[0];
        let args = cmd[1..].to_vec();
        // dbg!(&args);
        // let args = args.join(" ");

        let task = self.tasks.get(name).expect("No task found");
        let mut file = NamedTempFile::new()?;
        file.write(task.cmd.as_str().as_bytes())?;
        let path = file.path().to_str().unwrap();
        // let path = file.into_temp_path();
        // let cmd = Conf::replace_args(task.cmd.as_str(), &args);

        if env::var("RUST_LOG").is_ok() {
            dbg!(&cmd);
        }
        // println!("Running: {}", cmd.green());

        let mut cmd_args = vec![path];
        let mut arg_clone = args.clone();
        cmd_args.append(&mut arg_clone);

        let mut cmd = Command::new("/bin/sh");
        let mut env = self.env.clone();
        env.extend(task.env.clone());

        // this is to collect the creaed envs
        let mut parsed_env: IndexMap<String, String> = IndexMap::new();
        for (key, value) in env {
              let mut env_file = NamedTempFile::new()?;
            env_file.write(format!("echo {}", value.as_str()).as_str().as_bytes())?;
            // create a command to evaluate the env
            let path = env_file.path().to_str().unwrap();
            let mut env_cmd = Command::new("/bin/sh");
            let mut env_cmd_args = vec![path];
            let mut arg_clone = args.clone();
            env_cmd_args.append(&mut arg_clone);
            env_cmd.args(env_cmd_args);
            for (key, value) in &parsed_env {
                let env_val = env::var(key).unwrap_or(value.to_string());
                dbg!(&env_val);
                env_cmd.env(key, env_val);
            }
            let output = env_cmd.output()?;
            let output = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let trimmed_output = env::var(&key).unwrap_or(output);


            parsed_env.insert(
                String::from_str(key.as_str())?,
                String::from(&trimmed_output),
            );
            if env::var("RUST_LOG").is_ok() {
                dbg!(&key, &trimmed_output);
            }
            cmd.env(key, trimmed_output);
        }
        cmd.args(cmd_args);
        // dbg!(&cmd);

        // set the working directory
        if let Some(path) = &task.workdir {
            if !task.local {
                cmd.current_dir(path);
            }
        }

        // if running from command line, use prodcution and don't capture output
        if env::var("DOIT_PROD").is_ok() {
            cmd.spawn()?.wait()?;
            return Ok("".to_string());
        }
        // for testing, capture output
        let output: std::process::Output = cmd.output()?;
        let res = String::from_utf8_lossy(&output.stdout).to_string();
        println!("{}", res);
        Ok(res)
    }

    fn extend(&mut self, other: Conf) {
        self.tasks.extend(other.tasks);
        let mut env = self.env.clone();
        env.extend(other.env);
        self.env = env;
    }
}

#[derive(Deserialize, Debug)]
struct Task {
    cmd: String,
    #[serde(default)]
    env: IndexMap<String, String>,
    #[serde(default)]
    workdir: Option<PathBuf>,
    #[serde(default)]
    local: bool,
}

pub fn get_dofiles(wd: Option<PathBuf>) -> anyhow::Result<Conf> {
    let mut path;
    match wd {
        Some(wd) => {
            path = wd;
        }
        None => {
            path = env::current_dir()?;
        }
    }

    let mut conf: Conf = Default::default();
    loop {
        let subpath = path.join("do.yaml");
        if subpath.exists() {
            // the below code is meant to ensure that children override parents
            let text = fs::read_to_string(subpath)?;
            let mut new_conf: Conf = serde_yaml::from_str(&text)?;

            // assign workdir to all tasks that don't have one
            new_conf.tasks = new_conf
                .tasks
                .into_iter()
                .map(|(key, mut value)| {
                    if value.workdir == None {
                        value.workdir = Some(path.clone());
                        return (key, value);
                    }
                    (key, value)
                })
                .collect();
            new_conf.extend(conf);
            conf = new_conf;
        }
        if path.parent().is_none() {
            break;
        }
        path = path.parent().unwrap().to_path_buf();
    }
    if conf.tasks.is_empty() {
        anyhow::bail!("No do.yaml found");
    }

    Ok(conf)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::write;
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
    fn test_multiline_command() -> anyhow::Result<()> {
        let conf = serde_yaml::from_str::<Conf>(
            r#"
tasks:
    hello:
        cmd: |
            echo hello
            echo world"#,
        )?;
        let res = conf.exec(vec!["hello"])?;
        assert_eq!(res, "hello\nworld\n");
        Ok(())
    }

    #[test]
    fn test_global_env() -> anyhow::Result<()> {
        let text = r#"
env:
    NAME: "world"
tasks:
    hello:
        cmd: echo hello $NAME"#;
        let conf: Conf = serde_yaml::from_str(&text)?;
        assert_eq!(conf.exec(vec!["hello"])?, "hello world\n");
        Ok(())
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
        assert_eq!(conf.exec(vec!["hello"])?, "hello world\n");
        Ok(())
    }

    #[test]
    fn test_parnent_env_override() -> anyhow::Result<()> {
        let parent = r#"env:
    NAME: world
    "#;
        let child = r#"env:
    NAME: child
tasks:
    hello:
        cmd: echo hello $NAME"#;
        let mut conf: Conf = serde_yaml::from_str(&parent)?;

        let child_conf: Conf = serde_yaml::from_str(&child)?;

        conf.extend(child_conf);
        assert_eq!(conf.exec(vec!["hello"])?, "hello child\n");
        Ok(())
    }

    #[test]
    fn test_task_env_override() -> anyhow::Result<()> {
        let parent = r#"env:
    NAME: world
tasks:
    hello:
        cmd: echo hello $NAME
        env:
            NAME: task"#;
        let conf: Conf = serde_yaml::from_str(&parent)?;

        assert_eq!(conf.exec(vec!["hello"])?, "hello task\n");
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

    #[test]
    fn parent_and_local_tasks_combine() -> anyhow::Result<()> {
        let temp_dir = tempfile::tempdir()?;
        write(
            temp_dir.path().join("do.yaml"),
            r#"env:
    NAME: world
tasks:
    hello:
        cmd: echo hello""#,
        )?;

        fs::create_dir(temp_dir.path().join("child"))?;

        write(
            temp_dir.path().join("child/do.yaml"),
            r#"env:
    BOY: child
tasks:
    bye:
        cmd: echo bye"#,
        )?;

        let conf = get_dofiles(Some(temp_dir.path().join("child")))?;
        assert_eq!(conf.tasks.len(), 2);
        assert_eq!(conf.env.len(), 2);

        Ok(())
    }

    #[test]
    fn test_extra_args() -> anyhow::Result<()> {
        let conf = serde_yaml::from_str::<Conf>(
            r#"tasks:
    hello:
        cmd: echo hello $1"#,
        )?;
        assert_eq!(conf.exec(vec!["hello", "world"])?, "hello world\n");
        Ok(())
    }

    /// test that args are passed to env
    #[test]
    fn test_args_to_env() -> anyhow::Result<()> {
        let conf = serde_yaml::from_str::<Conf>(
            r#"tasks:
    hello:
        cmd: echo hello $ARGS
        env:
            ARGS: $1"#,
        )?;
        assert_eq!(conf.exec(vec!["hello", "world"])?, "hello world\n");
        Ok(())
    }

    #[test]
    fn test_evaluated_env() -> anyhow::Result<()> {
        let conf = serde_yaml::from_str::<Conf>(
            r#"tasks:
    hello:
        cmd: echo hello $NAME
        env:
            HIDDEN: dworld
            NAME: $HIDDEN"#,
        )?;
        assert_eq!(conf.exec(vec!["hello"])?, "hello dworld\n");
        Ok(())
    }

    #[test]
    fn test_hyphenated_args() -> anyhow::Result<()> {
        let conf = serde_yaml::from_str::<Conf>(
            r#"tasks:
    hello:
        cmd: echo hello $1"#,
        )?;
        assert_eq!(
            conf.exec(vec!["hello", "real-world"])?,
            "hello real-world\n"
        );
        Ok(())
    }

    #[test]
    fn test_multiple_args() -> anyhow::Result<()> {
        let conf = serde_yaml::from_str::<Conf>(
            r#"tasks:
    hello:
        cmd: echo hello $1 $2"#,
        )?;
        assert_eq!(
            conf.exec(vec!["hello", "real", "world"])?,
            "hello real world\n"
        );
        Ok(())
    }

    #[test]
    fn test_quoted_args() -> anyhow::Result<()> {
        let conf = serde_yaml::from_str::<Conf>(
            r#"tasks:
    hello:
        cmd: echo hello $1"#,
        )?;
        assert_eq!(
            conf.exec(vec!["hello", "real world"])?,
            "hello real world\n"
        );
        Ok(())
    }

    #[test]
    fn test_override_env() -> anyhow::Result<()> {
        env::set_var("NAME_TEST", "override");
        let conf = serde_yaml::from_str::<Conf>(
            r#"env:
    NAME: world
tasks:
    hello:
        cmd: echo hello $NAME_TEST"#,
        )?;
        assert_eq!(conf.exec(vec!["hello"])?, "hello override\n");
        env::remove_var("NAME_TEST");
        Ok(())
    }
}
