//! An aggregate invariant: the windows are sorted and none of them overlap.
//!
//! 1. The invariant is a relationship between elements, so the whole list is checked at once.
//! 2. `new` and `insert` are the only routes in. The `Vec` is private, so no caller can push.
//! 3. Readers get a slice. A sorted, non-overlapping slice supports a binary search.

use std::ops::Range;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// A sorted list of half-open windows that do not overlap.
#[derive(Clone, PartialEq, Eq, Debug, Default)] // std
#[derive(Serialize, Deserialize)] // serde
#[serde(try_from = "Vec<Range<u32>>", into = "Vec<Range<u32>>")]
pub struct Windows(Vec<Range<u32>>);

/// A window is empty, or two windows overlap.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum WindowsError {
    /// The window's end is at or before its start.
    #[error("window {0:?} is empty")]
    Empty(Range<u32>),
    /// The new window shares at least one point with a window already in the list.
    #[error("window {new:?} overlaps {existing:?}")]
    Overlap {
        /// The window that was rejected.
        new: Range<u32>,
        /// The window already in the list.
        existing: Range<u32>,
    },
}

impl Windows {
    /// Builds the list from windows in any order.
    ///
    /// # Errors
    ///
    /// An empty window or an overlapping pair rejects the whole list.
    pub fn new(windows: Vec<Range<u32>>) -> Result<Self, WindowsError> {
        let mut checked = Self::default();
        for window in windows {
            checked.insert(window)?;
        }
        Ok(checked)
    }

    /// Inserts one window in its sorted place.
    ///
    /// # Errors
    ///
    /// An empty window or an overlap is rejected, and the list is unchanged.
    pub fn insert(&mut self, window: Range<u32>) -> Result<(), WindowsError> {
        if window.is_empty() {
            return Err(WindowsError::Empty(window));
        }
        let at = self
            .0
            .partition_point(|existing| existing.start < window.start);
        // The neighbour before starts earlier, so only its end can reach into the new window.
        if let Some(before) = at.checked_sub(1).and_then(|i| self.0.get(i))
            && before.end > window.start
        {
            return Err(WindowsError::Overlap {
                new: window,
                existing: before.clone(),
            });
        }
        // The neighbour after starts at or later, so only the new window's end can reach it.
        if let Some(after) = self.0.get(at)
            && window.end > after.start
        {
            return Err(WindowsError::Overlap {
                new: window,
                existing: after.clone(),
            });
        }
        self.0.insert(at, window);
        Ok(())
    }

    /// The windows in order. A binary search on `start` is valid.
    #[must_use]
    pub fn as_slice(&self) -> &[Range<u32>] {
        &self.0
    }

    /// Whether `point` falls inside a window.
    #[must_use]
    pub fn contains(&self, point: u32) -> bool {
        let at = self.0.partition_point(|window| window.start <= point);
        at.checked_sub(1)
            .and_then(|i| self.0.get(i))
            .is_some_and(|window| window.contains(&point))
    }
}

impl TryFrom<Vec<Range<u32>>> for Windows {
    type Error = WindowsError;

    fn try_from(windows: Vec<Range<u32>>) -> Result<Self, WindowsError> {
        Self::new(windows)
    }
}

impl From<Windows> for Vec<Range<u32>> {
    fn from(windows: Windows) -> Self {
        windows.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_sorts_and_rejects_an_overlap() -> anyhow::Result<()> {
        let windows = Windows::new(vec![20..30, 0..10])?;
        assert_eq!(windows.as_slice(), &[0..10, 20..30]);
        // 5..25 shares 5..10 with the first window
        let overlap = Windows::new(vec![0..10, 20..30, 5..25]);
        assert_eq!(
            overlap,
            Err(WindowsError::Overlap {
                new: 5..25,
                existing: 0..10
            })
        );
        Ok(())
    }

    #[test]
    fn test_insert_leaves_the_list_unchanged_on_rejection() -> anyhow::Result<()> {
        let mut windows = Windows::new(vec![0..10, 20..30])?;
        let before = windows.clone();
        assert!(windows.insert(10..10).is_err());
        assert!(windows.insert(9..12).is_err());
        assert_eq!(windows, before);
        windows.insert(10..12)?;
        assert_eq!(windows.as_slice(), &[0..10, 10..12, 20..30]);
        Ok(())
    }

    #[test]
    fn test_json_route_runs_the_same_check() {
        let overlapping = r#"[{"start":0,"end":10},{"start":5,"end":6}]"#;
        assert!(serde_json::from_str::<Windows>(overlapping).is_err());
    }

    #[test]
    fn test_contains_uses_the_half_open_bound() -> anyhow::Result<()> {
        let windows = Windows::new(vec![0..10, 20..30])?;
        assert!(windows.contains(9), "9 is inside 0..10");
        assert!(!windows.contains(10), "10 is the excluded end of 0..10");
        assert!(windows.contains(20), "20 is the included start of 20..30");
        Ok(())
    }
}
