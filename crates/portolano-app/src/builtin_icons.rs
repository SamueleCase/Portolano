//! The builtin icons (SPEC-UI §4.7, DEC-029.4): their names, which are data
//! of the document (`Icon::Builtin { name }`), and the table that shows the
//! icons of the Royal library with them, since the images of Royal are not
//! copied (licence).

/// The names of the builtin icons, exactly those of DEC-029.4. The frontend
/// maps each one to an icon of its set; a test there checks that the two
/// lists match.
pub const BUILTIN_ICONS: [&str; 30] = [
    "server",
    "desktop",
    "laptop",
    "database",
    "terminal",
    "cloud",
    "network",
    "router",
    "globe",
    "key",
    "lock",
    "shield",
    "folder",
    "hard-drive",
    "cpu",
    "container",
    "box",
    "building",
    "home",
    "user",
    "users",
    "mail",
    "printer",
    "phone",
    "wifi",
    "git-branch",
    "code",
    "settings",
    "star",
    "tag",
];

/// Words of the names of the Royal library (`<set>/<category>/<name>`, such
/// as `Flat/Hardware/Server`) and the builtin icon that shows them. The
/// first word found wins, so the specific words come before the general
/// ones (`laptop` before `computer`, `users` before `user`).
const ROYAL_WORDS: &[(&str, &str)] = &[
    ("server", "server"),
    ("laptop", "laptop"),
    ("notebook", "laptop"),
    ("database", "database"),
    ("sql", "database"),
    ("terminal", "terminal"),
    ("console", "terminal"),
    ("shell", "terminal"),
    ("cloud", "cloud"),
    ("router", "router"),
    ("wifi", "wifi"),
    ("wireless", "wifi"),
    ("network", "network"),
    ("switch", "network"),
    ("globe", "globe"),
    ("world", "globe"),
    ("web", "globe"),
    ("internet", "globe"),
    ("key", "key"),
    ("lock", "lock"),
    ("firewall", "shield"),
    ("shield", "shield"),
    ("security", "shield"),
    ("folder", "folder"),
    ("disk", "hard-drive"),
    ("drive", "hard-drive"),
    ("storage", "hard-drive"),
    ("cpu", "cpu"),
    ("chip", "cpu"),
    ("processor", "cpu"),
    ("container", "container"),
    ("docker", "container"),
    ("box", "box"),
    ("package", "box"),
    ("building", "building"),
    ("office", "building"),
    ("company", "building"),
    ("home", "home"),
    ("house", "home"),
    ("users", "users"),
    ("group", "users"),
    ("user", "user"),
    ("person", "user"),
    ("mail", "mail"),
    ("email", "mail"),
    ("printer", "printer"),
    ("phone", "phone"),
    ("mobile", "phone"),
    ("git", "git-branch"),
    ("code", "code"),
    ("settings", "settings"),
    ("gear", "settings"),
    ("cog", "settings"),
    ("star", "star"),
    ("tag", "tag"),
    ("label", "tag"),
    ("desktop", "desktop"),
    ("monitor", "desktop"),
    ("computer", "desktop"),
    ("workstation", "desktop"),
];

/// Whether `name` is a builtin icon.
pub fn is_builtin(name: &str) -> bool {
    BUILTIN_ICONS.contains(&name)
}

/// The builtin icon that shows the Royal icon `name`, from the words of its
/// last segment, then of its category; `None` without a match (the node
/// then shows the icon of its type).
pub fn for_royal(name: &str) -> Option<&'static str> {
    let mut segments = name.rsplit('/');
    let last = segments.next().unwrap_or_default();
    let category = segments.next().unwrap_or_default();
    match_words(last).or_else(|| match_words(category))
}

fn match_words(text: &str) -> Option<&'static str> {
    let text = text.to_lowercase();
    ROYAL_WORDS
        .iter()
        .find(|(word, _)| text.contains(word))
        .map(|&(_, builtin)| builtin)
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn builtin_names_are_unique() {
        let unique: HashSet<&str> = BUILTIN_ICONS.into_iter().collect();

        assert_eq!(unique.len(), BUILTIN_ICONS.len());
    }

    #[test]
    fn every_royal_word_leads_to_a_builtin_icon() {
        for (word, builtin) in ROYAL_WORDS {
            assert!(is_builtin(builtin), "{word} → {builtin}");
        }
    }

    #[test]
    fn royal_name_is_matched_by_its_last_segment() {
        assert_eq!(for_royal("Flat/Hardware/Server"), Some("server"));
        assert_eq!(for_royal("Flat/Hardware/Monitor"), Some("desktop"));
        assert_eq!(
            for_royal("VMware Clarity/Technology/Laptop Computer"),
            Some("laptop")
        );
    }

    #[test]
    fn royal_name_falls_back_to_its_category() {
        assert_eq!(for_royal("Flat/Network/Hub 24"), Some("network"));
    }

    #[test]
    fn royal_name_without_a_known_word_has_no_builtin_icon() {
        assert_eq!(for_royal("Flat/Animals/Penguin"), None);
    }
}
