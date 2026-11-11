//! Tests of path syntax and resolution, sharing one sample tree.

use super::*;
use crate::testutil::{connection, document, folder};

const CLIENTI: Uuid = Uuid::from_u128(0x1c000000_0000_4000_8000_000000000001);
const ACME: Uuid = Uuid::from_u128(0xacacac00_0000_4000_8000_000000000002);
const WEB01: Uuid = Uuid::from_u128(0xabcdef01_0000_4000_8000_000000000003);
const WEB02: Uuid = Uuid::from_u128(0xabcdef02_0000_4000_8000_000000000004);
const SLASHED: Uuid = Uuid::from_u128(0x51000000_0000_4000_8000_000000000005);
const BACKSLASHED: Uuid = Uuid::from_u128(0xb5000000_0000_4000_8000_000000000006);
const PROD_UPPER: Uuid = Uuid::from_u128(0x90000000_0000_4000_8000_000000000007);
const DB_UPPER: Uuid = Uuid::from_u128(0x9d000000_0000_4000_8000_000000000008);
const PROD_LOWER: Uuid = Uuid::from_u128(0x91000000_0000_4000_8000_000000000009);
const DB_LOWER: Uuid = Uuid::from_u128(0x9e000000_0000_4000_8000_00000000000a);
const LINUX_FIRST: Uuid = Uuid::from_u128(0x11000000_0000_4000_8000_00000000000b);
const LINUX_SECOND: Uuid = Uuid::from_u128(0x12000000_0000_4000_8000_00000000000c);
const ONLY_IN_SECOND: Uuid = Uuid::from_u128(0x13000000_0000_4000_8000_00000000000d);
const NAMED_LIKE_AN_ID: Uuid = Uuid::from_u128(0x14000000_0000_4000_8000_00000000000f);
const MISSING: Uuid = Uuid::from_u128(0xf0000000_0000_4000_8000_00000000000e);

/// ```text
/// /Clienti/ACME/web01            (abcdef01…)
/// /Clienti/ACME/web02            (abcdef02…)
/// /Clienti/a\/b
/// /Clienti/back\\slash
/// /Prod/db
/// /prod/db
/// /Linux, /Linux/only-in-second  (two sibling folders named Linux)
/// ```
fn sample() -> Document {
    document(vec![
        folder(
            CLIENTI,
            "Clienti",
            vec![
                folder(
                    ACME,
                    "ACME",
                    vec![connection(WEB01, "web01"), connection(WEB02, "web02")],
                ),
                connection(SLASHED, "a/b"),
                connection(BACKSLASHED, "back\\slash"),
            ],
        ),
        folder(PROD_UPPER, "Prod", vec![connection(DB_UPPER, "db")]),
        folder(PROD_LOWER, "prod", vec![connection(DB_LOWER, "db")]),
        folder(LINUX_FIRST, "Linux", vec![]),
        folder(
            LINUX_SECOND,
            "Linux",
            vec![connection(ONLY_IN_SECOND, "only-in-second")],
        ),
    ])
}

fn invalid_reason(result: Result<impl std::fmt::Debug>) -> String {
    match result {
        Err(Error::InvalidPath { reason, .. }) => reason,
        other => panic!("expected InvalidPath, got {other:?}"),
    }
}

mod depth;
mod resolution;
mod syntax;
