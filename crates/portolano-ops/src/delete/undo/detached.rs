//! What a forced deletion changed outside the deleted subtree, and how an
//! undo gives it back without overwriting what changed since (DEC-031.2):
//! a credential goes back only if it is still "ask at launch" as the
//! deletion left it, and a setting only where its value is still the one
//! the deletion wrote. The settings are never replaced as a whole.

use portolano_core::{CredentialSpec, Document, Node};
use serde_json::Value;
use uuid::Uuid;

use crate::nodes::{credential_spec, credential_spec_mut};

/// A step of the path of a value inside the settings.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Step {
    Key(String),
    Index(usize),
}

/// One value of the settings changed by the deletion.
#[derive(Debug, Clone)]
struct SettingChange {
    path: Vec<Step>,
    before: Value,
    after: Value,
}

/// The changes the deletion made to one node.
#[derive(Debug, Clone)]
pub(super) struct Detached {
    node: Uuid,
    /// The credential before and after, if it changed.
    credential: Option<(CredentialSpec, CredentialSpec)>,
    settings: Vec<SettingChange>,
}

/// What an undo did to a node changed by the deletion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Restored {
    /// Everything went back.
    All,
    /// Something changed again since the deletion, and stayed as it is.
    Kept,
}

impl Detached {
    /// The changes to node `id` between `before` and `after` the detachment.
    pub(super) fn record(before: &Document, after: &Document, id: Uuid) -> Detached {
        let (old, new) = (before.find(id), after.find(id));
        let credential = match (old.and_then(credential_spec), new.and_then(credential_spec)) {
            (Some(old), Some(new)) if old != new => Some((old.clone(), new.clone())),
            _ => None,
        };
        let mut settings = Vec::new();
        if let (Some(old), Some(new)) = (settings_of(old), settings_of(new)) {
            diff(&mut Vec::new(), &old, &new, &mut settings);
        }
        Detached {
            node: id,
            credential,
            settings,
        }
    }

    /// The node changed by the deletion.
    pub(super) fn node(&self) -> Uuid {
        self.node
    }

    /// Gives back what is still as the deletion left it. The node is
    /// touched only if something went back.
    pub(super) fn restore(self, doc: &mut Document) -> Restored {
        // A node deleted meanwhile has nothing to give back, and nothing
        // was kept of it either.
        let Some(node) = doc.find_mut(self.node) else {
            return Restored::All;
        };
        let mut restored = false;
        let mut kept = false;
        if let Some((before, after)) = self.credential {
            match credential_spec_mut(node) {
                Some(slot) if *slot == after => {
                    *slot = before;
                    restored = true;
                }
                _ => kept = true,
            }
        }
        for change in self.settings {
            let current = node
                .as_connection_mut()
                .and_then(|connection| value_at(&mut connection.settings, &change.path));
            match current {
                Some(value) if *value == change.after => {
                    *value = change.before;
                    restored = true;
                }
                _ => kept = true,
            }
        }
        if restored {
            node.touch();
        }
        if kept { Restored::Kept } else { Restored::All }
    }
}

fn settings_of(node: Option<&Node>) -> Option<Value> {
    node.and_then(Node::as_connection)
        .map(|connection| Value::Object(connection.settings.clone()))
}

/// The paths where `before` and `after` differ, as deep as both have the
/// same shape.
fn diff(path: &mut Vec<Step>, before: &Value, after: &Value, changes: &mut Vec<SettingChange>) {
    if before == after {
        return;
    }
    match (before, after) {
        (Value::Object(old), Value::Object(new))
            if old.len() == new.len() && old.keys().all(|key| new.contains_key(key)) =>
        {
            for (key, value) in old {
                path.push(Step::Key(key.clone()));
                diff(path, value, &new[key], changes);
                path.pop();
            }
        }
        (Value::Array(old), Value::Array(new)) if old.len() == new.len() => {
            for (index, (old, new)) in old.iter().zip(new).enumerate() {
                path.push(Step::Index(index));
                diff(path, old, new, changes);
                path.pop();
            }
        }
        _ => changes.push(SettingChange {
            path: path.clone(),
            before: before.clone(),
            after: after.clone(),
        }),
    }
}

/// The value at `path` in `settings`, if it is there.
fn value_at<'a>(
    settings: &'a mut serde_json::Map<String, Value>,
    path: &[Step],
) -> Option<&'a mut Value> {
    let (Step::Key(first), rest) = path.split_first()? else {
        return None;
    };
    let mut value = settings.get_mut(first)?;
    for step in rest {
        value = match step {
            Step::Key(key) => value.as_object_mut()?.get_mut(key)?,
            Step::Index(index) => value.as_array_mut()?.get_mut(*index)?,
        };
    }
    Some(value)
}
