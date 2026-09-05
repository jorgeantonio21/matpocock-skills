//! An authoritative transition result: the mutation returns the facts it established.
//!
//! 1. `remove_group` holds the state while it runs, so it alone knows which members went with the group.
//! 2. The outcome carries them. A second lookup after the call finds nothing, or a new group under the same id.
//! 3. The event is built from the outcome, not from a second query.

use std::collections::HashMap;

/// A group of members.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct GroupId(pub u32);

/// A member of one group.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct MemberId(pub u32);

/// Why a group was not removed.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum RejectReason {
    /// No group has this id.
    UnknownGroup(GroupId),
}

/// What `remove_group` did.
#[must_use = "the removed members must reach the caller"]
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum RemoveOutcome {
    /// The group and its members are gone.
    Removed {
        /// The members that were in the group, in their stored order.
        members: Vec<MemberId>,
        /// How many groups the registry still holds.
        groups_left: usize,
    },
    /// Nothing changed.
    Rejected(RejectReason),
}

/// The groups and their members.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Registry {
    groups: HashMap<GroupId, Vec<MemberId>>,
}

impl Registry {
    /// Adds `member` to `group`, creating the group on its first member.
    pub fn add(&mut self, group: GroupId, member: MemberId) {
        self.groups.entry(group).or_default().push(member);
    }

    /// Removes `group` and every member in it. The outcome says which members those were.
    pub fn remove_group(&mut self, group: GroupId) -> RemoveOutcome {
        let Some(members) = self.groups.remove(&group) else {
            return RemoveOutcome::Rejected(RejectReason::UnknownGroup(group));
        };
        RemoveOutcome::Removed {
            members,
            groups_left: self.groups.len(),
        }
    }

    /// The members of `group`, or `None` when no group has this id.
    #[must_use]
    pub fn members(&self, group: GroupId) -> Option<&[MemberId]> {
        self.groups.get(&group).map(Vec::as_slice)
    }
}

/// The event a caller emits after a removal.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct GroupRemoved {
    /// The removed group.
    pub group: GroupId,
    /// The members that left with it.
    pub members: Vec<MemberId>,
}

/// Removes `group` and builds the event from the outcome, the one record of what was removed.
pub fn remove_and_announce(registry: &mut Registry, group: GroupId) -> Option<GroupRemoved> {
    match registry.remove_group(group) {
        RemoveOutcome::Removed {
            members,
            groups_left: _,
        } => Some(GroupRemoved { group, members }),
        RemoveOutcome::Rejected(RejectReason::UnknownGroup(_)) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_outcome_carries_exactly_the_removed_members() {
        let mut registry = Registry::default();
        registry.add(GroupId(1), MemberId(10));
        registry.add(GroupId(1), MemberId(11));
        registry.add(GroupId(2), MemberId(20));
        let event = remove_and_announce(&mut registry, GroupId(1));
        assert_eq!(
            event,
            Some(GroupRemoved {
                group: GroupId(1),
                members: vec![MemberId(10), MemberId(11)]
            })
        );
        // a second lookup no longer sees the members the event names
        assert_eq!(registry.members(GroupId(1)), None);
        assert_eq!(registry.members(GroupId(2)), Some(&[MemberId(20)][..]));
    }

    #[test]
    fn test_removing_an_unknown_group_changes_nothing() {
        let mut registry = Registry::default();
        registry.add(GroupId(2), MemberId(20));
        let before = registry.clone();
        assert_eq!(
            registry.remove_group(GroupId(1)),
            RemoveOutcome::Rejected(RejectReason::UnknownGroup(GroupId(1)))
        );
        assert_eq!(registry, before);
    }
}
