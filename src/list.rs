use crate::config::Conf;
use colored::*;

pub trait List {
  fn list_commands(&self);
}

impl List for Conf {
  fn list_commands(&self) {
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
}