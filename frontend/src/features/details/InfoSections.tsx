import type { JsonValue, NodeDetailsView_Serialize, NodeRef } from "../../bindings";
import { Card } from "../../components/Card";
import { GroupHeader } from "../../components/GroupHeader";
import { KeyValueGrid, type KeyValueRow } from "../../components/KeyValueGrid";
import type { VaultBackend } from "../../components/SecretBadge";
import { visibleText } from "../../components/visibleText";
import { interpolate, plural } from "../../i18n/format";
import { texts } from "../../i18n/it";
import { readablePath } from "../tree/treeModel";
import { useGoToNode } from "../tree/useGoToNode";
import styles from "./Details.module.css";
import { MultilineText } from "./MultilineText";

const dates = new Intl.DateTimeFormat("it-IT", { dateStyle: "long" });

/** An imported value Portolano does not map, as text. */
function jsonText(value: JsonValue): string {
  return typeof value === "string" ? value : JSON.stringify(value);
}

/**
 * Who uses the node (inventario §5.2): the nodes that refer to it, and the
 * connections that inherit it from a folder that does; each row goes to the
 * node.
 */
export function UsageSection({ details }: { details: NodeDetailsView_Serialize }) {
  const t = texts.details.usage;
  const goTo = useGoToNode();
  const rows: [NodeRef, string][] = [
    ...details.usage.direct.map((node): [NodeRef, string] => [node, t.direct]),
    ...details.usage.inherited.map((node): [NodeRef, string] => [node, t.inherited]),
  ];
  if (rows.length === 0) return null;
  return (
    <section className={styles.section} aria-label={t.heading}>
      <GroupHeader>{t.heading}</GroupHeader>
      <p className={styles.usageCount}>{plural(rows.length, t.usedBy)}</p>
      <ul className={styles.usageList}>
        {rows.map(([node, relation]) => (
          <li key={`${node.id}-${relation}`}>
            <button type="button" className={styles.usageRow} onClick={() => goTo(node.id)}>
              <span>{visibleText(readablePath(node.path))}</span>
              <span className={styles.relation}>{relation}</span>
            </button>
          </li>
        ))}
      </ul>
    </section>
  );
}

/**
 * "Informazioni" (inventario §5.2): description, notes, the custom properties
 * (a secret one only by its state), where the node was imported from, and the
 * imported fields that Portolano keeps without mapping them, read-only.
 */
export function InfoSection({
  details,
  backend,
}: {
  details: NodeDetailsView_Serialize;
  backend: VaultBackend;
}) {
  const t = texts.details.info;
  const secretState = (name: string): string => {
    const slot = details.secrets.find(
      (secret) => typeof secret.slot === "object" && secret.slot.property === name,
    );
    switch (slot?.state) {
      case "present":
        return t.secretStates.present[backend];
      case "absent":
        return t.secretStates.absent;
      case "vaultLocked":
        return t.secretStates.vaultLocked;
      default:
        return t.secretStates.none;
    }
  };
  const rows: KeyValueRow[] = [];
  if (details.description) {
    rows.push({
      key: "description",
      label: t.description,
      value: visibleText(details.description),
    });
  }
  if (details.notes) {
    rows.push({
      key: "notes",
      label: t.notes,
      value: (
        <span className={styles.notes}>
          <MultilineText text={details.notes} />
        </span>
      ),
    });
  }
  for (const property of details.properties) {
    rows.push({
      key: `property-${property.name}`,
      label: property.name,
      mono: true,
      value: property.secret
        ? interpolate(t.secretProperty, { state: secretState(property.name) })
        : visibleText(property.text ?? ""),
    });
  }
  if (details.source) {
    const { system, file, importedAt } = details.source;
    const systems: Readonly<Record<string, string>> = t.sourceSystems;
    rows.push({
      key: "source",
      label: t.source,
      value: visibleText(
        interpolate(t.sourceValue, {
          system: systems[system] ?? system,
          file: file ?? "",
          date: dates.format(new Date(importedAt)),
        }),
      ),
    });
  }
  const extra = Object.entries(details.extra);
  if (rows.length === 0 && extra.length === 0) return null;
  return (
    <section className={styles.section} aria-label={t.heading}>
      <GroupHeader>{t.heading}</GroupHeader>
      {rows.length > 0 && <KeyValueGrid rows={rows} />}
      {extra.length > 0 && (
        <Card className={styles.extra} aria-label={t.extra} role="group">
          <h4 className={styles.extraTitle}>{t.extra}</h4>
          <KeyValueGrid
            rows={extra.map(([key, value]) => ({
              key,
              label: key,
              mono: true,
              value: visibleText(jsonText(value)),
            }))}
          />
        </Card>
      )}
    </section>
  );
}

/**
 * A node Portolano does not support (inventario §5.3): what it is, why it is
 * kept read-only, and the original fragment as text, never as markup.
 */
export function UnsupportedNotice({ details }: { details: NodeDetailsView_Serialize }) {
  const t = texts.details.unsupported;
  const original = details.unsupported;
  if (original === null) return null;
  return (
    <>
      <Card tinted className={styles.unsupported}>
        {t.before}
        <strong className={styles.sourceType}>{visibleText(original.sourceType)}</strong>
        {t.after}
      </Card>
      <pre className={styles.raw} aria-label={t.raw} tabIndex={0}>
        <MultilineText text={original.raw} />
      </pre>
    </>
  );
}
