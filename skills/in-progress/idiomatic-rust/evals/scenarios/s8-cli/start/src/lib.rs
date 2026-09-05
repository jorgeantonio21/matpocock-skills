#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Capacity { Unlimited, Bounded(u8) }

pub fn load(bytes: &[u8]) -> Result<Capacity, &'static str> {
    match bytes {
        [1 | 2, value] => Ok(Capacity::Bounded((*value).max(1))),
        _ => Err("invalid record"),
    }
}

pub fn save_new(value: u8) -> Result<[u8; 2], &'static str> {
    if value == 0 { return Err("new records require a nonzero capacity"); }
    Ok([2, value])
}

pub fn label(title: &str, body: &str) -> String { format!("{title}: {body}") }
