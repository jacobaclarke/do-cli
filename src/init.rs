use colored::*;
use std::path::Path;

use crate::config::Conf;

pub fn init() {
    let conf = Conf::default();
    let text = serde_yaml::to_string(&conf).unwrap();
    let path = Path::new("do.yaml");
    if path.exists() {
        println!("{}", "No file created: do.yaml already exists".green());
    } else {
        println!("{}", "Creating do.yaml".green());
        if let Err(e) = std::fs::write(path, text) {
            println!("{}", e);
        }
    }
}
