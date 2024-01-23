use crate::config::Conf;
use colored::*;

pub trait List {
    fn list_commands(&self);
}

impl List for Conf {
    fn list_commands(&self) {
        println!("Environment:\n--------------------");
        self.env
            .iter()
            .for_each(|(key, value)| println!("{}={}", key.red(), value.blue()));

        println!("\nAvailable commands:\n--------------------");

        self.tasks
            .iter()
            .filter(|(_, t)| !t.hidden)
            .for_each(|(_, task)| {
                task.env
                    .iter()
                    .for_each(|(key, value)| println!("  {}={}", key.red(), value.blue()));

                task.cmd
                    .lines()
                    .for_each(|row| println!("  {}", row.blue()));
            });
    }
}
