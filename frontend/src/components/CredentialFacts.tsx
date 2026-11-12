import { texts } from "../i18n/it";
import styles from "./CredentialPreview.module.css";
import type { ResolvedCredential } from "./credentialPreviewData";
import { SecretBadge, type VaultBackend } from "./SecretBadge";
import { visibleText } from "./visibleText";

/** User, domain, password and key of a resolved credential, as a small grid. */
export function CredentialFacts({
  credential,
  backend,
}: {
  credential: ResolvedCredential;
  backend: VaultBackend;
}) {
  const t = texts.components.credentialPreview;
  return (
    <dl className={styles.facts}>
      <dt>{t.user}</dt>
      <dd className={styles.mono}>{credential.user ? visibleText(credential.user) : t.noDomain}</dd>
      {credential.domain && (
        <>
          <dt>{t.domain}</dt>
          <dd className={styles.mono}>{visibleText(credential.domain)}</dd>
        </>
      )}
      {credential.password && (
        <>
          <dt>{t.password}</dt>
          <dd>
            <SecretBadge state={credential.password} backend={backend} variant="text" />
          </dd>
        </>
      )}
      {credential.key && (
        <>
          <dt>{t.key}</dt>
          <dd>{credential.key}</dd>
        </>
      )}
    </dl>
  );
}
