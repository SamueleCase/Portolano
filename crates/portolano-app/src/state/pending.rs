//! The deletions that can still be undone (SPEC-UI §5.4: "Eliminato ·
//! Annulla" for 10 s, then the secrets are deleted).

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Duration;

use portolano_ops::delete::DeletedSubtree;
use uuid::Uuid;

use super::recover;

/// How long a deletion can be undone: the life of the toast "Annulla".
pub const UNDO_WINDOW: Duration = Duration::from_secs(10);

/// The identifier of an undoable deletion, given to the frontend.
pub type UndoId = Uuid;

/// A deletion that can still be undone, and the document it belongs to.
pub struct PendingDelete {
    /// The document file.
    pub document: PathBuf,
    /// What [`portolano_ops::delete::restore_deleted`] needs.
    pub deleted: DeletedSubtree,
}

/// The deletions that can still be undone, by id.
#[derive(Default)]
pub struct PendingDeletes {
    entries: Mutex<HashMap<UndoId, PendingDelete>>,
}

impl PendingDeletes {
    /// Records a deletion from `document` and returns its id.
    pub fn insert(&self, document: &Path, deleted: DeletedSubtree) -> UndoId {
        let id = Uuid::new_v4();
        recover(self.entries.lock()).insert(
            id,
            PendingDelete {
                document: document.to_owned(),
                deleted,
            },
        );
        id
    }

    /// Removes deletion `id`, to undo or to finish it; `None` if it was
    /// already undone or finished.
    pub fn take(&self, id: UndoId) -> Option<PendingDelete> {
        recover(self.entries.lock()).remove(&id)
    }

    /// Removes every deletion.
    pub fn take_all(&self) -> Vec<PendingDelete> {
        recover(self.entries.lock())
            .drain()
            .map(|(_, pending)| pending)
            .collect()
    }
}
