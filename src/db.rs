use anyhow::Result;
use chrono::NaiveDate;
use core::panic;
use serde::{Deserialize, Serialize};
use serde_json;
use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

use crate::task::{parse_date, Task};

#[derive(Debug, Serialize, Deserialize)]
pub struct Db {
    pub db_path: Box<Path>,
    pub tasks: Vec<Task>,
    // config TK
}

impl Default for Db {
    fn default() -> Self {
        let home = match home::home_dir() {
            Some(path) if !path.as_os_str().is_empty() => path,
            _ => panic!("Unable to get your home dir!"),
        };
        let default_db_path = home.join(".shittd.json");
        Db {
            db_path: default_db_path.into_boxed_path(),
            tasks: Vec::new(),
        }
    }
}
impl Db {
    pub fn init(&mut self) -> Result<()> {
        // Opens or creates a file at the object's path
        if self.db_path.exists() {
            self.open()
        } else {
            let mut db_file = File::create(self.db_path.clone())?;
            db_file.write_all(b"{}")?; // empty json
            Ok(())
        }
    }

    pub fn update(&mut self) -> Result<()> {
        #[derive(Serialize, Deserialize, Debug)]
        struct OldFormat {
            id: u8,
            name: String,
            complete: bool,
        }
        let read_file =
            fs::read_to_string(self.db_path.as_os_str()).expect("Unable to read in data file");
        let prior_tasks: Vec<OldFormat> =
            serde_json::from_str(&read_file).expect("Invalid data file.");
        self.tasks = prior_tasks
            .iter()
            .map(|t| Task {
                id: t.id,
                name: t.name.clone(),
                complete: t.complete,
                ..Default::default()
            })
            .collect();
        Ok(())
    }

    pub fn open(&mut self) -> Result<()> {
        let read_file =
            fs::read_to_string(self.db_path.as_os_str()).expect("Unable to read in data file");
        self.tasks = serde_json::from_str(&read_file)?;
        Ok(())
    }

    pub fn save(self) -> Result<()> {
        fs::write(
            self.db_path,
            serde_json::to_string_pretty(&self.tasks).unwrap(),
        )
        .expect("Unable to write data file.");
        Ok(())
    }

    pub fn get_next_id(&self) -> Option<u8> {
        let max_id = &self.tasks.iter().max_by_key(|task| task.id)?;
        Some(max_id.id + 1)
    }

    pub fn insert_task(&mut self, task_name: String, date: NaiveDate) {
        let next_id = self.get_next_id().unwrap_or(1);

        self.tasks.push(Task {
            id: next_id,
            name: task_name,
            date,
            complete: false,
        });
        self.order_tasks()
    }

    pub fn push_tasks(&mut self, tasks_to_finish: Vec<u8>, date: Option<String>) {
        if date.is_some() {
            let new_date = parse_date(date.unwrap()).expect("Unable to parse date");
            self.tasks
                .iter_mut()
                .filter(|task| tasks_to_finish.contains(&task.id))
                .for_each(|t| t.date = new_date);
        } else {
            self.tasks
                .iter_mut()
                .filter(|task| tasks_to_finish.contains(&task.id))
                .for_each(|t| t.push());
        }
        self.order_tasks()
    }

    pub fn finish_tasks(&mut self, tasks_to_finish: Vec<u8>) {
        self.tasks
            .iter_mut()
            .filter(|task| tasks_to_finish.contains(&task.id))
            .for_each(|t| t.finish());

        self.order_tasks()
    }

    pub fn remove_finished_tasks(&mut self) {
        self.tasks.retain(|t| !t.complete);
        self.order_tasks()
    }

    // Orders tasks by complete and then ID
    pub fn order_tasks(&mut self) {
        self.tasks.sort_by_key(|t| (t.complete, t.date));
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Days, Local};

    use super::*;

    #[test]
    fn test_order() {
        let first = Task {
            name: "Incomplete Soonest".to_string(),
            ..Default::default()
        };
        let second = Task {
            name: "Incomplete later".to_string(),
            date: Local::now().date_naive() + Days::new(3),
            ..Default::default()
        };
        let third = Task {
            name: "Complete Sooner".to_string(),
            complete: true,
            ..Default::default()
        };
        let fourth = Task {
            name: "Complete later".to_string(),
            date: Local::now().date_naive() + Days::new(3),
            complete: true,
            ..Default::default()
        };

        let mut db = Db {
            tasks: vec![fourth.clone(), third.clone(), second.clone(), first.clone()],
            ..Default::default()
        };
        db.order_tasks();
        let mut tasks_iter = db.tasks.iter();
        assert_eq!(tasks_iter.next().unwrap().name, first.name);
        assert_eq!(tasks_iter.next().unwrap().name, second.name);
        assert_eq!(tasks_iter.next().unwrap().name, third.name);
        assert_eq!(tasks_iter.next().unwrap().name, fourth.name);
    }
}
