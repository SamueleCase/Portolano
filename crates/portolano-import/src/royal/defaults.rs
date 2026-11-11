//! Default values of Royal fields that are omitted from the XML
//! (`docs/royal-format.md` §3).
//!
//! Royal omits every field at its default. Flags default to `False` and
//! optional values (ports, sizes, scale) to "not set", which the mapping
//! expresses with `None`; an absent `CredentialMode` means "inherit"
//! (SPEC §8.3.5, DEC-006). The constants below are the defaults that need a
//! value.

/// `PositionNr` of an object that omits it.
pub(super) const POSITION_NR: i64 = 0;
