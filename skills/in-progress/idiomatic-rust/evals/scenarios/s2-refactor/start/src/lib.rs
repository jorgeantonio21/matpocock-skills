use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::fs;
use std::time::SystemTime;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Note {
    pub id: u64,
    /// 0 means the note has no parent.
    pub parent: u64,
    pub title: String,
    pub body: String,
    pub tags: Vec<String>,
    /// "open" or "archived".
    pub status: String,
    pub updated_at: SystemTime,
}

#[derive(Debug)]
pub enum StoreError {
    Parse(String),
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StoreError::Parse(msg) => write!(f, "parse error: {msg}"),
        }
    }
}

impl Error for StoreError {}

#[derive(Debug)]
pub struct Store {
    notes: Vec<Note>,
    index: HashMap<u64, usize>,
    next_id: u64,
}

impl Store {
    pub fn new() -> Self {
        Self {
            notes: Vec::new(),
            index: HashMap::new(),
            next_id: 1,
        }
    }

    pub fn add_note(&mut self, title: &String, body: &String, parent: u64, now: SystemTime) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.notes.push(Note {
            id,
            parent,
            title: title.clone(),
            body: body.clone(),
            tags: Vec::new(),
            status: "open".to_string(),
            updated_at: now,
        });
        self.index.insert(id, self.notes.len() - 1);
        id
    }

    pub fn add_tag(&mut self, id: u64, tag: &str) {
        let i = self.index[&id];
        self.notes[i].tags.push(tag.to_string());
    }

    pub fn get_notes(&self) -> &Vec<Note> {
        &self.notes
    }

    pub fn find(&self, id: u64) -> Option<&Note> {
        match self.index.get(&id) {
            Some(i) => Some(&self.notes[*i]),
            None => None,
        }
    }

    pub fn titles(&self) -> Vec<String> {
        let mut out = Vec::new();
        for i in 0..self.notes.len() {
            out.push(self.notes[i].title.clone());
        }
        out
    }

    pub fn set_status(&mut self, id: u64, status: &str) {
        let i = self.index[&id];
        match status {
            "open" | "archived" => self.notes[i].status = status.to_string(),
            _ => panic!("unknown status {status}"),
        }
    }

    pub fn count_by_tag(&self, tag: &str, include_archived: bool) -> usize {
        self.notes
            .iter()
            .filter(|n| n.tags.iter().any(|t| t == tag))
            .filter(|n| include_archived || n.status != "archived")
            .count()
    }

    pub fn archive_stale(&mut self) -> usize {
        let now = SystemTime::now();
        let notes = self.notes.clone();
        let mut archived = 0;
        for note in notes {
            if note.status == "open" && self.is_stale(&note, now) {
                self.set_status(note.id, "archived");
                archived += 1;
            }
        }
        archived
    }

    fn is_stale(&self, note: &Note, now: SystemTime) -> bool {
        use std::time::Duration;
        // We consider a note stale after a week. This was decided in the design meeting
        // in March; the old implementation used 30 days.
        let limit = Duration::from_secs(7 * 24 * 60 * 60);
        now.duration_since(note.updated_at)
            .map(|age| age > limit)
            .unwrap_or(false)
    }

    pub fn newest(&self) -> Note {
        self.notes
            .iter()
            .max_by_key(|n| n.updated_at)
            .unwrap()
            .clone()
    }
}

pub fn load(path: &str, now: SystemTime) -> Result<Store, Box<dyn Error>> {
    let text = fs::read_to_string(path).unwrap();
    let mut store = Store::new();
    for (line_no, line) in text.lines().enumerate() {
        let Some((title, body)) = line.split_once('|') else {
            return Err(Box::new(StoreError::Parse(format!(
                "line {} has no '|' separator: {}",
                line_no + 1,
                line
            ))));
        };
        store.add_note(&title.to_string(), &body.to_string(), 0, now);
    }
    Ok(store)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    const T0: SystemTime = SystemTime::UNIX_EPOCH;

    fn at(secs: u64) -> SystemTime {
        T0 + Duration::from_secs(secs)
    }

    #[test]
    fn add_then_find_returns_the_note() {
        let mut store = Store::new();
        let id = store.add_note(&"a".to_string(), &"body".to_string(), 0, at(1));
        assert_eq!(store.find(id).map(|n| n.title.as_str()), Some("a"));
        assert!(store.find(999).is_none());
    }

    #[test]
    fn titles_follow_insertion_order() {
        let mut store = Store::new();
        store.add_note(&"first".to_string(), &"".to_string(), 0, at(1));
        store.add_note(&"second".to_string(), &"".to_string(), 0, at(2));
        assert_eq!(store.titles(), vec!["first", "second"]);
        assert_eq!(store.get_notes().len(), 2);
    }

    #[test]
    fn archived_notes_leave_the_tag_count_unless_asked_for() {
        let mut store = Store::new();
        let a = store.add_note(&"a".to_string(), &"".to_string(), 0, at(1));
        let b = store.add_note(&"b".to_string(), &"".to_string(), 0, at(2));
        store.add_tag(a, "work");
        store.add_tag(b, "work");
        store.set_status(b, "archived");
        assert_eq!(store.count_by_tag("work", false), 1);
        assert_eq!(store.count_by_tag("work", true), 2);
    }

    #[test]
    fn archive_stale_archives_open_notes_older_than_a_week() {
        let mut store = Store::new();
        let eight_days_ago = SystemTime::now() - Duration::from_secs(8 * 24 * 60 * 60);
        let old = store.add_note(&"old".to_string(), &"".to_string(), 0, eight_days_ago);
        let fresh = store.add_note(&"fresh".to_string(), &"".to_string(), 0, SystemTime::now());
        assert_eq!(store.archive_stale(), 1);
        assert_eq!(store.find(old).unwrap().status, "archived");
        assert_eq!(store.find(fresh).unwrap().status, "open");
        assert_eq!(store.archive_stale(), 0);
    }

    #[test]
    fn newest_is_the_most_recently_updated() {
        let mut store = Store::new();
        store.add_note(&"older".to_string(), &"".to_string(), 0, at(10));
        store.add_note(&"newer".to_string(), &"".to_string(), 0, at(20));
        assert_eq!(store.newest().title, "newer");
    }

    #[test]
    fn load_reads_one_note_per_line() {
        let path = std::env::temp_dir().join(format!("notes-ok-{}.txt", std::process::id()));
        fs::write(&path, "a|body a\nb|body b\n").unwrap();
        let store = load(path.to_str().unwrap(), at(1)).unwrap();
        assert_eq!(store.titles(), vec!["a", "b"]);
        assert_eq!(store.find(2).unwrap().body, "body b");
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn load_rejects_a_line_without_a_separator() {
        let path = std::env::temp_dir().join(format!("notes-bad-{}.txt", std::process::id()));
        fs::write(&path, "a|body a\nno separator here\n").unwrap();
        let err = load(path.to_str().unwrap(), at(1)).unwrap_err();
        assert!(err.to_string().contains("line 2"));
        fs::remove_file(path).unwrap();
    }
}
