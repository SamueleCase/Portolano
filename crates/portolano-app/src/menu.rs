//! The native menu, in Italian (SPEC-UI §4.9). Each item of the app sends a
//! [`MenuAction`] event; the interface performs the action, as for the same
//! button or shortcut. The standard items (edit, window, quit) are those of
//! the system.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tauri::menu::{
    AboutMetadata, IsMenuItem, Menu, MenuEvent, MenuItem, MenuItemBuilder, PredefinedMenuItem,
    Submenu, SubmenuBuilder,
};
use tauri::{AppHandle, Manager, Runtime};
use tauri_specta::Event;

use crate::dto::file_name;
use crate::events::emit;
use crate::state::AppState;

/// What a menu item asks the interface to do.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum MenuCommand {
    /// Preferenze…
    Preferences,
    /// Nuovo documento
    NewDocument,
    /// Apri…
    OpenDocument,
    /// A recent document.
    OpenRecent {
        /// Its path, as `recent_documents` lists it.
        path: String,
    },
    /// Importa…
    Import,
    /// Chiudi tab
    CloseTab,
    /// Nuova connessione
    NewConnection,
    /// Nuova cartella
    NewFolder,
    /// Nuova credenziale
    NewCredential,
    /// Connetti
    Connect,
    /// Modifica
    Edit,
    /// Duplica
    Duplicate,
    /// Elimina
    Delete,
    /// Cerca
    Search,
    /// Valida
    Validate,
    /// Densità normale
    DensityNormal,
    /// Densità compatta
    DensityCompact,
}

/// A menu item was chosen.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type, Event)]
pub struct MenuAction {
    /// What to do.
    pub command: MenuCommand,
}

/// An item of the app: its command, its label and its shortcut.
struct Item {
    id: &'static str,
    command: MenuCommand,
    label: &'static str,
    shortcut: Option<&'static str>,
}

const fn item(
    id: &'static str,
    command: MenuCommand,
    label: &'static str,
    shortcut: Option<&'static str>,
) -> Item {
    Item {
        id,
        command,
        label,
        shortcut,
    }
}

/// Prefix of the ids of the recent documents, followed by their index.
const RECENT_PREFIX: &str = "recent:";

static PREFERENCES: Item = item(
    "preferences",
    MenuCommand::Preferences,
    "Preferenze…",
    Some("CmdOrCtrl+,"),
);

static FILE_ITEMS: [Item; 2] = [
    item(
        "new-document",
        MenuCommand::NewDocument,
        "Nuovo documento",
        None,
    ),
    item(
        "open-document",
        MenuCommand::OpenDocument,
        "Apri…",
        Some("CmdOrCtrl+O"),
    ),
];

static FILE_TAIL: [Item; 2] = [
    item(
        "import",
        MenuCommand::Import,
        "Importa…",
        Some("CmdOrCtrl+I"),
    ),
    item(
        "close-tab",
        MenuCommand::CloseTab,
        "Chiudi tab",
        Some("CmdOrCtrl+W"),
    ),
];

static NODE_ITEMS: [Item; 7] = [
    item(
        "new-connection",
        MenuCommand::NewConnection,
        "Nuova connessione",
        Some("CmdOrCtrl+N"),
    ),
    item(
        "new-folder",
        MenuCommand::NewFolder,
        "Nuova cartella",
        Some("CmdOrCtrl+Shift+N"),
    ),
    item(
        "new-credential",
        MenuCommand::NewCredential,
        "Nuova credenziale",
        None,
    ),
    item(
        "connect",
        MenuCommand::Connect,
        "Connetti",
        Some("CmdOrCtrl+Enter"),
    ),
    item("edit", MenuCommand::Edit, "Modifica", Some("CmdOrCtrl+E")),
    item(
        "duplicate",
        MenuCommand::Duplicate,
        "Duplica",
        Some("CmdOrCtrl+D"),
    ),
    item(
        "delete",
        MenuCommand::Delete,
        "Elimina",
        Some("CmdOrCtrl+Backspace"),
    ),
];

static VIEW_ITEMS: [Item; 2] = [
    item("search", MenuCommand::Search, "Cerca", Some("CmdOrCtrl+F")),
    item(
        "validate",
        MenuCommand::Validate,
        "Valida",
        Some("CmdOrCtrl+Shift+V"),
    ),
];

static DENSITY_ITEMS: [Item; 2] = [
    item(
        "density-normal",
        MenuCommand::DensityNormal,
        "Normale",
        None,
    ),
    item(
        "density-compact",
        MenuCommand::DensityCompact,
        "Compatta",
        None,
    ),
];

/// Every item of the app.
fn items() -> impl Iterator<Item = &'static Item> {
    std::iter::once(&PREFERENCES)
        .chain(&FILE_ITEMS)
        .chain(&FILE_TAIL)
        .chain(&NODE_ITEMS)
        .chain(&VIEW_ITEMS)
        .chain(&DENSITY_ITEMS)
}

/// The command of the item `id`, given the recent documents of the menu.
pub fn command_for(id: &str, recents: &[PathBuf]) -> Option<MenuCommand> {
    if let Some(index) = id.strip_prefix(RECENT_PREFIX) {
        let recent = recents.get(index.parse::<usize>().ok()?)?;
        return Some(MenuCommand::OpenRecent {
            path: recent.display().to_string(),
        });
    }
    items()
        .find(|item| item.id == id)
        .map(|item| item.command.clone())
}

/// Builds the menu with `recents` and makes it the menu of the app.
///
/// # Errors
/// Errors of the menu of the system.
pub fn install<R: Runtime>(app: &AppHandle<R>, recents: &[PathBuf]) -> tauri::Result<()> {
    let menu = build(app, recents)?;
    app.set_menu(menu)?;
    Ok(())
}

/// Sends the command of a chosen item to the frontend.
pub fn on_event<R: Runtime>(app: &AppHandle<R>, event: &MenuEvent) {
    let recents = app.state::<AppState>().ui_preferences().recent_documents;
    if let Some(command) = command_for(event.id().as_ref(), &recents) {
        emit(app, MenuAction { command });
    }
}

fn build<R: Runtime>(app: &AppHandle<R>, recents: &[PathBuf]) -> tauri::Result<Menu<R>> {
    let about = PredefinedMenuItem::about(
        app,
        Some("Informazioni su Portolano"),
        Some(AboutMetadata {
            name: Some("Portolano".to_owned()),
            ..AboutMetadata::default()
        }),
    )?;
    let separator = || PredefinedMenuItem::separator(app);
    let app_menu = SubmenuBuilder::new(app, "Portolano")
        .item(&about)
        .item(&separator()?)
        .item(&built(app, &PREFERENCES)?)
        .item(&separator()?)
        .item(&PredefinedMenuItem::quit(app, Some("Esci da Portolano"))?)
        .build()?;
    let file = SubmenuBuilder::new(app, "File")
        .items(&as_items(&built_all(app, &FILE_ITEMS)?))
        .item(&recent_menu(app, recents)?)
        .item(&separator()?)
        .items(&as_items(&built_all(app, &FILE_TAIL)?))
        .build()?;
    let edit = SubmenuBuilder::new(app, "Modifica")
        .undo_with_text("Annulla")
        .redo_with_text("Ripeti")
        .separator()
        .cut_with_text("Taglia")
        .copy_with_text("Copia")
        .paste_with_text("Incolla")
        .select_all_with_text("Seleziona tutto")
        .build()?;
    let node = SubmenuBuilder::new(app, "Nodo")
        .items(&as_items(&built_all(app, &NODE_ITEMS)?))
        .build()?;
    let density = SubmenuBuilder::new(app, "Densità")
        .items(&as_items(&built_all(app, &DENSITY_ITEMS)?))
        .build()?;
    let view = SubmenuBuilder::new(app, "Vista")
        .items(&as_items(&built_all(app, &VIEW_ITEMS)?))
        .item(&density)
        .build()?;
    let window = SubmenuBuilder::new(app, "Finestra")
        .item(&PredefinedMenuItem::minimize(app, Some("Riduci a icona"))?)
        .item(&PredefinedMenuItem::maximize(app, Some("Ingrandisci"))?)
        .item(&PredefinedMenuItem::fullscreen(
            app,
            Some("Schermo intero"),
        )?)
        .build()?;
    Menu::with_items(app, &[&app_menu, &file, &edit, &node, &view, &window])
}

/// "Documenti recenti", with the file names of `recents`.
fn recent_menu<R: Runtime>(app: &AppHandle<R>, recents: &[PathBuf]) -> tauri::Result<Submenu<R>> {
    let items = recents
        .iter()
        .enumerate()
        .map(|(index, path)| {
            MenuItemBuilder::with_id(format!("{RECENT_PREFIX}{index}"), recent_label(path))
                .build(app)
        })
        .collect::<tauri::Result<Vec<_>>>()?;
    SubmenuBuilder::new(app, "Documenti recenti")
        .items(&as_items(&items))
        .enabled(!recents.is_empty())
        .build()
}

/// The label of a recent document: its file name and its directory, which
/// tells apart two documents with the same name.
fn recent_label(path: &Path) -> String {
    match path.parent() {
        Some(dir) if !dir.as_os_str().is_empty() => {
            format!("{} — {}", file_name(path), dir.display())
        }
        _ => file_name(path),
    }
}

fn built<R: Runtime>(app: &AppHandle<R>, item: &Item) -> tauri::Result<MenuItem<R>> {
    let builder = MenuItemBuilder::with_id(item.id, item.label);
    match item.shortcut {
        Some(shortcut) => builder.accelerator(shortcut),
        None => builder,
    }
    .build(app)
}

fn built_all<R: Runtime>(app: &AppHandle<R>, items: &[Item]) -> tauri::Result<Vec<MenuItem<R>>> {
    items.iter().map(|item| built(app, item)).collect()
}

fn as_items<R: Runtime>(items: &[MenuItem<R>]) -> Vec<&dyn IsMenuItem<R>> {
    items
        .iter()
        .map(|item| item as &dyn IsMenuItem<R>)
        .collect()
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn every_item_has_its_own_id_and_command() {
        let ids: HashSet<&str> = items().map(|item| item.id).collect();
        let commands: Vec<MenuCommand> = items().map(|item| item.command.clone()).collect();

        assert_eq!(ids.len(), items().count());
        for (index, command) in commands.iter().enumerate() {
            assert!(!commands[..index].contains(command), "{command:?}");
        }
    }

    #[test]
    fn item_id_leads_to_its_command() {
        assert_eq!(command_for("connect", &[]), Some(MenuCommand::Connect));
        assert_eq!(command_for("sconosciuto", &[]), None);
    }

    #[test]
    fn recent_item_leads_to_its_document() {
        let recents = vec![
            PathBuf::from("/Users/utente-di-prova/a.json"),
            PathBuf::from("/Users/utente-di-prova/b.json"),
        ];

        assert_eq!(
            command_for("recent:1", &recents),
            Some(MenuCommand::OpenRecent {
                path: "/Users/utente-di-prova/b.json".to_owned()
            })
        );
        assert_eq!(command_for("recent:2", &recents), None);
    }

    #[test]
    fn shortcuts_are_those_of_the_specification() {
        let shortcut = |id: &str| {
            items()
                .find(|item| item.id == id)
                .and_then(|item| item.shortcut)
        };

        assert_eq!(shortcut("preferences"), Some("CmdOrCtrl+,"));
        assert_eq!(shortcut("open-document"), Some("CmdOrCtrl+O"));
        assert_eq!(shortcut("import"), Some("CmdOrCtrl+I"));
        assert_eq!(shortcut("close-tab"), Some("CmdOrCtrl+W"));
        assert_eq!(shortcut("new-connection"), Some("CmdOrCtrl+N"));
        assert_eq!(shortcut("new-folder"), Some("CmdOrCtrl+Shift+N"));
        assert_eq!(shortcut("connect"), Some("CmdOrCtrl+Enter"));
        assert_eq!(shortcut("edit"), Some("CmdOrCtrl+E"));
        assert_eq!(shortcut("duplicate"), Some("CmdOrCtrl+D"));
        assert_eq!(shortcut("delete"), Some("CmdOrCtrl+Backspace"));
        assert_eq!(shortcut("search"), Some("CmdOrCtrl+F"));
        assert_eq!(shortcut("validate"), Some("CmdOrCtrl+Shift+V"));
    }
}
