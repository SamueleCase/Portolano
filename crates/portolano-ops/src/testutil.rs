//! Fixtures shared by the unit tests of the crate (SPEC §11.3). Hosts are
//! under `example.com` and users are fake (SPEC §1.3).

use std::path::{Path, PathBuf};

use portolano_core::vault::{FileVault, KdfParams};
use portolano_core::{MasterPasswordSource, Secret, VaultError};

/// The only master password allowed in tests (SPEC §1.3).
pub(crate) const TEST_MASTER_PASSWORD: &str = "test-only-master-password";

/// The cheapest Argon2id costs, so that file vaults open quickly.
const TEST_KDF_PARAMS: KdfParams = KdfParams {
    m_cost_kib: 8,
    t_cost: 1,
    p_cost: 1,
};

/// Always answers [`TEST_MASTER_PASSWORD`].
pub(crate) struct TestPassword;

impl MasterPasswordSource for TestPassword {
    fn master_password(&self, _creating: bool) -> Result<Secret, VaultError> {
        Ok(Secret::new(TEST_MASTER_PASSWORD.to_owned()))
    }
}

/// Creates an empty file vault with cheap KDF costs in `dir` and returns its
/// path; reopening it uses the costs stored in the file.
pub(crate) fn fast_file_vault(dir: &Path) -> PathBuf {
    let path = dir.join("vault.json");
    FileVault::with_kdf_params(&path, &TestPassword, TEST_KDF_PARAMS).unwrap();
    path
}
