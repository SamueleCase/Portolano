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

/// The builtin icon of the first word of the table that is a whole word of
/// `text`: `Clock` is not `lock`, `Mailbox` is not `box`.
fn match_words(text: &str) -> Option<&'static str> {
    let words = words_of(text);
    ROYAL_WORDS
        .iter()
        .find(|(word, _)| words.iter().any(|candidate| candidate == word))
        .map(|&(_, builtin)| builtin)
}

/// The lowercase words of `text`, split at anything that is not a letter
/// or a digit and where a lowercase letter meets an uppercase one
/// (`HardDrive` gives `hard` and `drive`).
fn words_of(text: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();
    let mut previous_lowercase = false;
    for c in text.chars() {
        let boundary = !c.is_alphanumeric() || (c.is_uppercase() && previous_lowercase);
        if boundary && !current.is_empty() {
            words.push(std::mem::take(&mut current));
        }
        if c.is_alphanumeric() {
            current.extend(c.to_lowercase());
        }
        previous_lowercase = c.is_lowercase();
    }
    if !current.is_empty() {
        words.push(current);
    }
    words
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
    fn only_whole_words_match() {
        assert_eq!(for_royal("Flat/Objects/Clock"), None);
        assert_eq!(for_royal("Flat/Objects/Keyboard"), None);
        assert_eq!(for_royal("Flat/Objects/Mailbox"), None);
    }

    #[test]
    fn words_joined_in_camel_case_are_split() {
        assert_eq!(for_royal("Flat/Hardware/ExternalDrive"), Some("hard-drive"));
    }

    #[test]
    fn royal_name_without_a_known_word_has_no_builtin_icon() {
        assert_eq!(for_royal("Flat/Animals/Penguin"), None);
    }
}
