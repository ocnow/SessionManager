use std::{fs::File, io::Read, os::unix::fs::FileExt, str::FromStr, time::Duration};

use serde::{Deserialize, Serialize};

const JSON_DB_FILE_PATH: &str = "resources/sessionDB.json";

#[derive(Serialize, Deserialize, Debug)]
struct SkillEntry {
    name: String,
    total_time_spent: usize,
}

pub fn get_list_of_skills(JSON_DB_FILE_NAME: &str) -> Vec<String> {
    let mut dbFile = File::open(JSON_DB_FILE_NAME).unwrap();
    let mut data = String::new();
    dbFile.read_to_string(&mut data);
    let mut skillEntrySet: Vec<SkillEntry> = Vec::new();
    if (!data.trim().is_empty()) {
        skillEntrySet = serde_json::from_str(&data).unwrap();
        println!("SkillSet we got size : {}", skillEntrySet.len());
    } else {
        println!("got the empty string");
    }

    return skillEntrySet.into_iter().map(|x| x.name).collect();
}
