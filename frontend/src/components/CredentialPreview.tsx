import { interpolate } from "../i18n/format";
import { texts } from "../i18n/it";
import { Button } from "./Button";
import { CredentialFacts } from "./CredentialFacts";
import styles from "./CredentialPreview.module.css";
import type { CredentialPreviewActions, CredentialPreviewData } from "./credentialPreviewData";
import { GroupHeader } from "./GroupHeader";
import { NodeTypeIcon } from "./icons/NodeTypeIcon";
import { RouteChain } from "./RouteChain";
import { SecretBadge, type VaultBackend } from "./SecretBadge";
import { TextField } from "./TextField";
import { visibleText } from "./visibleText";

interface CredentialPreviewProps {
  data: CredentialPreviewData;
  backend: VaultBackend;
  actions?: CredentialPreviewActions | undefined;
}

const t = texts.components.credentialPreview;

function Body({ data, backend, actions = {} }: CredentialPreviewProps) {
  switch (data.mode) {
    case "inherit":
      return data.source ? (
        <>
          <RouteChain
            variant="inherit"
            label={t.heading}
            stops={data.source.chain.map((label) => ({ label }))}
          />
          <p className={styles.text}>{interpolate(t.inherit, { node: visibleText(data.node) })}</p>
          <CredentialFacts credential={data.source.credential} backend={backend} />
        </>
      ) : (
        <>
          <h3 className={styles.title}>{t.inheritNoneTitle}</h3>
          <p className={styles.text}>
            {interpolate(t.inheritNone, { node: visibleText(data.node) })}
          </p>
        </>
      );
    case "none":
      return (
        <>
          <h3 className={styles.title}>{t.noneTitle}</h3>
          <p className={styles.text}>{t.none}</p>
        </>
      );
    case "ask":
      return (
        <>
          <h3 className={styles.title}>{t.askTitle}</h3>
          <p className={styles.text}>{t.ask}</p>
        </>
      );
    case "inline":
      return (
        <>
          <p className={styles.text}>{t.inline}</p>
          <div className={styles.pair}>
            <TextField
              label={t.user}
              mono
              value={data.user}
              onChange={(event) => actions.onInlineChange?.("user", event.target.value)}
            />
            <TextField
              label={t.domain}
              mono
              value={data.domain}
              onChange={(event) => actions.onInlineChange?.("domain", event.target.value)}
            />
          </div>
          <div className={styles.secret}>
            <span className={styles.secretLabel}>{t.password}</span>
            <SecretBadge state={data.password} backend={backend} />
            <Button variant="ghost" onClick={actions.onSetPassword}>
              {data.password === "absent" ? t.setPassword : t.replacePassword}
            </Button>
          </div>
        </>
      );
    case "linked":
      return (
        <>
          <div className={styles.linked}>
            <NodeTypeIcon kind="credential" />
            {data.credential ? (
              <span className={styles.linkedName}>
                <strong>{visibleText(data.credential.name)}</strong>
                <span className={styles.path}>{visibleText(data.credential.path)}</span>
              </span>
            ) : (
              <span className={styles.linkedName}>{t.noLinked}</span>
            )}
            <Button variant="small" onClick={actions.onChooseCredential}>
              {t.choose}
            </Button>
          </div>
          <p className={styles.text}>{t.linked}</p>
          {data.credential && (
            <CredentialFacts credential={data.credential.resolved} backend={backend} />
          )}
        </>
      );
    case "byName":
      return (
        <>
          <div className={styles.nameField}>
            <TextField
              label={t.byNameField}
              mono
              value={data.name}
              onChange={(event) => actions.onNameChange?.(event.target.value)}
            />
          </div>
          {data.match.kind === "found" && (
            <div className={styles.match}>
              <span className={styles.matchLabel}>{t.byNameMatch}</span>
              <span className={styles.matchChip}>{visibleText(data.match.path)}</span>
            </div>
          )}
          {data.match.kind === "notFound" && (
            <p className={styles.problem} role="status">
              {t.byNameNotFound}
            </p>
          )}
          {data.match.kind === "ambiguous" && (
            <div role="status" className={styles.problemBox}>
              <p className={styles.problem}>{t.byNameAmbiguous}</p>
              <ul className={styles.paths}>
                {data.match.paths.map((path) => (
                  <li key={path}>{visibleText(path)}</li>
                ))}
              </ul>
            </div>
          )}
          <p className={styles.text}>{t.byName}</p>
          {data.match.kind === "found" && (
            <CredentialFacts credential={data.match.resolved} backend={backend} />
          )}
        </>
      );
  }
}

/**
 * "Da dove arriverà" (inventario §4.24, §5.6): what the chosen mode means
 * for this node, with the states the mock did not draw (ambiguity 23): no
 * inherited credential, name not found, ambiguous name.
 */
export function CredentialPreview(props: CredentialPreviewProps) {
  return (
    <section key={props.data.mode} className={styles.preview} aria-label={t.heading}>
      <GroupHeader>{t.heading}</GroupHeader>
      <Body {...props} />
    </section>
  );
}
