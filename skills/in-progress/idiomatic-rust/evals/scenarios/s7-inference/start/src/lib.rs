use std::collections::VecDeque;

#[derive(Debug, PartialEq, Eq)]
pub struct Request { pub id: u64, pub tokens: u32, pub children: Vec<u64> }
#[derive(Debug, PartialEq, Eq)]
pub struct Removal { pub children: Vec<u64>, pub remaining: usize }
#[derive(Default)]
pub struct Scheduler { max_new_tokens: u32, queue: VecDeque<Request> }

impl Scheduler {
    pub fn set_max_new_tokens(&mut self, max: u32) { self.max_new_tokens = max; }
    pub fn admit(&mut self, request: Request) -> Result<(), Request> {
        if request.tokens == 0 || request.tokens > self.max_new_tokens {
            return Err(request);
        }
        self.queue.push_back(request);
        Ok(())
    }
    pub fn next_accepted(&mut self) -> Option<Request> {
        while let Some(request) = self.queue.pop_front() {
            if request.tokens <= self.max_new_tokens { return Some(request); }
        }
        None
    }
    pub fn remove(&mut self, id: u64) -> Option<Removal> {
        let index = self.queue.iter().position(|request| request.id == id)?;
        self.queue.remove(index)?;
        let children = self.queue.iter().find(|request| request.id == id)
            .map(|request| request.children.clone()).unwrap_or_default();
        Some(Removal { children, remaining: self.queue.len() })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn admitted_request_runs() {
        let mut scheduler = Scheduler::default();
        scheduler.set_max_new_tokens(10);
        scheduler.admit(Request { id: 1, tokens: 3, children: vec![] }).unwrap();
        assert_eq!(scheduler.next_accepted().unwrap().id, 1);
    }
}
