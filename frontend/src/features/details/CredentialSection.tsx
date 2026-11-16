import type {
  CredentialView_Serialize,
  KeyView,
  NodeDetailsView_Serialize,
  SecretState as BackendSecretState,
} from "../../bindings";
import { Card } from "../../components/Card";
import { ModeChip } from "../../components/ModeChip";
import { RouteChain } from "../../components/RouteChain";
import { SecretBadge, type SecretState, type VaultBackend } from "../../components/SecretBadge";
import { visibleText } from "../../components/visibleText";
import { interpolate } from "../../i18n/format";
import { texts } from "../../i18n/it";
import { credentialOrigin } from "./credentialOrigin";
import styles from "./Details.module.css";

interface CredentialSectionProps {
  details: NodeDetailsView_Serialize;
  backend: VaultBackend;
}

/** The state of a secret as the badge shows it; `undefined` when none is set. */
function badgeState(state: BackendSecretState, locked: boolean): SecretState | undefined {
  if (state === "not_set") return undefined;
  return locked ? "locked" : state;
}

/** The private key in words (inventario §5.2: "Nel portachiavi, con passphrase"). */
function keyText(
  credential: CredentialView_Serialize,
  backend: VaultBackend,
  locked: boolean,
): string {
  const t = texts.details.credential;
  const key: KeyView | null = credential.private_key;
  const base =
    key === null
      ? t.keyNone
      : key.kind === "file"
        ? interpolate(t.keyFile, { path: key.path })
        : locked
          ? texts.components.secretBadge.locked
          : key.state === "present"
            ? texts.components.secretBadge.present[backend]
            : texts.components.secretBadge.absent;
  return key !== null && credential.passphrase !== "not_set"
    ? interpolate(t.withPassphrase, { key: base })
    : base;
}

/**
 * "Credenziale effettiva" (inventario §5.2): where it comes from, with the
 * chain of `resolve_trace` when inherited, and its user, domain, password and
 * key, as states only (SPEC-UI §1.1). A credential that cannot be resolved
 * says why (ambiguity 23).
 */
export function CredentialSection({ details, backend }: CredentialSectionProps) {
  const t = texts.details.credential;
  const origin = credentialOrigin(details);
  if (origin === null) return null;
  const effective = details.effective;
  // Without a credential ("Nessuna", "Chiedi all'avvio") there are no data to show.
  const end = details.trace?.end.kind;
  const hasData = details.kind === "credential" || end === "credential" || end === "inline";
  const credential = hasData && effective?.kind === "resolved" ? effective.credential : null;
  const locked = details.vaultLocked;
  const password = credential ? badgeState(credential.password, locked) : undefined;

  return (
    <Card tinted large className={styles.credential} role="group" aria-label={t.heading}>
      <div className={styles.credentialHead}>
        <h3 className={styles.sectionTitle}>{t.heading}</h3>
        <ModeChip variant="detail">{origin.mode}</ModeChip>
        {origin.source !== "" && (
          <span className={styles.origin}>{visibleText(origin.source)}</span>
        )}
      </div>
      {origin.chain && (
        <RouteChain
          variant="inherit"
          label={t.chain}
          stops={origin.chain.map((label) => ({ label }))}
        />
      )}
      {origin.problem !== undefined && (
        <p className={styles.problem}>{visibleText(origin.problem)}</p>
      )}
      {effective?.kind === "unresolved" && origin.problem === undefined && (
        <p className={styles.problem}>{visibleText(effective.reason)}</p>
      )}
      {credential && (
        <dl className={styles.facts}>
          <div>
            <dt>{t.user}</dt>
            <dd className={styles.mono}>
              {credential.username ? visibleText(credential.username) : t.nothing}
            </dd>
          </div>
          <div>
            <dt>{t.domain}</dt>
            <dd className={styles.mono}>
              {credential.domain ? visibleText(credential.domain) : t.nothing}
            </dd>
          </div>
          <div>
            <dt>{t.password}</dt>
            <dd>{password ? <SecretBadge state={password} backend={backend} /> : t.nothing}</dd>
          </div>
          <div>
            <dt>{t.key}</dt>
            <dd className={styles.keyText}>{visibleText(keyText(credential, backend, locked))}</dd>
          </div>
        </dl>
      )}
    </Card>
  );
}
