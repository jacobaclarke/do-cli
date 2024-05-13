use crate::config::{Conf, Task};
use anyhow::anyhow;
use indexmap::IndexMap;
use rayon::prelude::*;
use std::{env, fs, io::Write, path::PathBuf, process::Command, str::FromStr};
use tempfile::NamedTempFile;

/// Adds abtility to execute commands from a configuration file
pub trait Exec {
    fn exec_single(
        &self,
        task: &Task,
        cmd: usize,
        command_args: Vec<&str>,
    ) -> anyhow::Result<String>;
    fn exec(&self, cmd: Vec<&str>) -> anyhow::Result<String>;
}

impl Exec for Conf {
    /// Executes a single command from a configuration file
    fn exec_single(
        &self,
        task: &Task,
        cmd: usize,
        command_args: Vec<&str>,
    ) -> anyhow::Result<String> {
        // write the task to a temp file to be executed
        let mut file = NamedTempFile::new()?;
        file.write_all(task.cmd[cmd].as_bytes())?;
        let path = file.path().to_str().expect("Unable to write a temp file");

        if env::var("RUST_LOG").is_ok() {
            dbg!(&cmd);
        }

        let mut cmd_args = vec![path];
        let mut arg_clone = command_args.clone();
        cmd_args.append(&mut arg_clone);

        let mut cmd = Command::new("/bin/bash");
        let mut env = self.env.clone();
        env.extend(task.env.clone());

        // this is to collect the creaed envs
        let mut parsed_env: IndexMap<String, String> = IndexMap::new();
        for (key, value) in env {
            let mut env_file = NamedTempFile::new()?;
            env_file.write_all(format!("echo {}", value.as_str()).as_str().as_bytes())?;
            // create a command to evaluate the env
            let path = env_file.path().to_str().unwrap();
            let mut env_cmd = Command::new("/bin/bash");
            let mut env_cmd_args = vec![path];
            let mut arg_clone = command_args.clone();
            env_cmd_args.append(&mut arg_clone);
            env_cmd.args(env_cmd_args);
            for (key, value) in &parsed_env {
                let env_val = env::var(key).unwrap_or(value.to_string());
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

    /// execute a shell command and return the output. Will also run the subcommands parallel
    fn exec(&self, cmd: Vec<&str>) -> anyhow::Result<String> {
        let command_name = cmd[0];
        let command_args = cmd[1..].to_vec();

        // retrive task from config
        let task = self
            .tasks
            .get(command_name)
            .ok_or(anyhow!("Task not found"))?;

        // TODO Add matches here to check for args
        (0..task.cmd.len())
            .collect::<Vec<usize>>()
            .into_par_iter()
            .map(|x| self.exec_single(task, x, command_args.clone()))
            .collect()
    }
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
            let mut new_conf: Conf = serde_yaml::from_str(&text)
                .map_err(|e| anyhow!("Failed to parse config: {}", e))?;

            // assign workdir to all tasks that don't have one
            new_conf.tasks = new_conf
                .tasks
                .into_iter()
                .map(|(key, mut value)| {
                    value.workdir = Some(value.workdir.unwrap_or(path.clone()));
                    (key, value)
                })
                .collect();
            new_conf.extend(conf);
            conf = new_conf;
        }
        match path.parent() {
            Some(p) => path = p.to_path_buf(),
            None => break,
        }
    }
    if conf.tasks.is_empty() {
        Err(anyhow::anyhow!("No tasks found."))
    } else {
        Ok(conf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::write;

    #[test]
    fn test_does_not_panic_when_missing() {
        let conf = Conf::default();
        assert!(conf.exec(vec!["nonexistent"]).is_err());
    }

    #[test]
    fn test_parallel_execution() -> anyhow::Result<()> {
        let text = r#"
tasks:
    hello:
        cmd: 
            - echo hello
            - echo world"#;
        let conf: Conf = serde_yaml::from_str(text)?;
        assert_eq!(conf.exec(vec!["hello"])?, "hello\nworld\n");
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
        let conf: Conf = serde_yaml::from_str(text)?;
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
        let mut conf: Conf = serde_yaml::from_str(parent)?;

        let child_conf: Conf = serde_yaml::from_str(child)?;

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
        let conf: Conf = serde_yaml::from_str(parent)?;

        assert_eq!(conf.exec(vec!["hello"])?, "hello task\n");
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
}
