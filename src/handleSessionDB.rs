use std::{
    fs::{File, OpenOptions},
    io::{self, BufWriter, Read, Seek, SeekFrom, Write},
    os::unix::fs::FileExt,
    str::FromStr,
    time::Duration,
};

use serde::{Deserialize, Serialize};

pub const JSON_DB_FILE_PATH: &str = "resources/sessionDB.json";

#[derive(Serialize, Deserialize, Debug)]
pub struct SkillEntry {
    pub name: String,
    pub total_time_spent: usize,
}

fn vector_contain_ignore_case(vec: Vec<String>, target: &str) -> bool {
    vec.into_iter()
        .any(|s| s.to_lowercase().eq_ignore_ascii_case(target))
}

pub fn get_list_of_skills(JSON_DB_FILE_NAME: &str) -> Vec<String> {
    let mut dbFile = File::open(JSON_DB_FILE_NAME).unwrap();
    let mut data = String::new();
    dbFile.read_to_string(&mut data);
    let mut skillEntrySet: Vec<SkillEntry> = Vec::new();
    if (!data.trim().is_empty()) {
        skillEntrySet = serde_json::from_str(&data).unwrap();
        // println!("SkillSet we got size : {}", skillEntrySet.len());
    } else {
        // println!("got the empty string");
    }

    return skillEntrySet.into_iter().map(|x| x.name).collect();
}

pub fn add_skill(JSON_DB_FILE_NAME: &str, skill_name: &str) -> io::Result<()> {
    let mut db_file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(JSON_DB_FILE_NAME)?;
    // let mut dbFile = File::open(JSON_DB_FILE_NAME).unwrap();
    let mut data = String::new();
    db_file.read_to_string(&mut data)?;

    let mut skill_entry_set: Vec<SkillEntry> = Vec::new();
    if !data.trim().is_empty() {
        skill_entry_set = serde_json::from_str(&data)?;
    }

    if !vector_contain_ignore_case(
        skill_entry_set.iter().map(|x| x.name.clone()).collect(),
        &skill_name,
    ) {
        skill_entry_set.push(SkillEntry {
            name: skill_name.to_string(),
            total_time_spent: 0,
        });
    }

    println!("{}", skill_entry_set.len());

    db_file.set_len(0)?;
    db_file.seek(SeekFrom::Start(0))?;
    db_file.write_all(serde_json::to_string(&skill_entry_set)?.as_bytes())?;

    Ok(())
}
