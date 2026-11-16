import { GuardedButton } from "../../app/GuardedButton";
import { useWindowActions } from "../../app/windowActions";
import { Button } from "../../components/Button";
import { IconTile } from "../../components/IconTile";
import { isConnection, type NodeKind } from "../../components/nodeKind";
import { Tag } from "../../components/Tag";
import { visibleText } from "../../components/visibleText";
import type { NodeDetailsView_Serialize } from "../../bindings";
import { plural } from "../../i18n/format";
import { texts } from "../../i18n/it";
import { useWorkspace } from "../../state/workspace";
import { iconChoice } from "../tree/treeModel";
import styles from "./Details.module.css";

interface DetailHeaderProps {
  details: NodeDetailsView_Serialize;
  kind: NodeKind;
  /** All the descendants of a folder: its summary, "N elementi". */
  descendants: number;
}

/**
 * The head of the detail (inventario §5.2): the icon tile with the color, the
 * type, the name with the star and the tags, the summary, and the actions. An
 * unsupported node is read-only: no "Modifica" (NOTE.md).
 */
export function DetailHeader({ details, kind, descendants }: DetailHeaderProps) {
  const t = texts.details;
  const actions = useWindowActions();
  const commandShown = useWorkspace((state) => state.commandShown);
  const toggleCommand = useWorkspace((state) => state.toggleCommand);
  const connection = isConnection(kind);
  const summary =
    kind === "folder" ? plural(descendants, texts.components.tree.descendants) : details.summary;

  return (
    <header className={styles.header}>
      <IconTile kind={kind} icon={iconChoice(details.icon)} color={details.color} />
      <div className={styles.identity}>
        <span className={styles.type}>{texts.components.nodeKinds[kind]}</span>
        <h2 className={styles.name}>
          <span>{visibleText(details.name)}</span>
          {details.favorite && (
            <span className={styles.star} role="img" aria-label={texts.components.favorite}>
              ★
            </span>
          )}
          {details.tags.length > 0 && (
            <span className={styles.tags} aria-label={t.tags}>
              {details.tags.map((tag) => (
                <Tag key={tag} name={tag} variant="detail" />
              ))}
            </span>
          )}
        </h2>
        {summary !== "" && <p className={styles.summary}>{visibleText(summary)}</p>}
      </div>
      <div role="group" aria-label={t.actions.label} className={styles.actions}>
        {connection && (
          <GuardedButton
            variant="primary"
            hint={t.actions.connect}
            shortcut={texts.app.shortcuts.connect}
            onClick={() => actions.connect(details.id)}
          >
            {t.actions.connect}
          </GuardedButton>
        )}
        {kind !== "unsupported" && (
          <GuardedButton
            reason={actions.offers("editor") ? undefined : texts.app.reasons.unavailable}
            onClick={() => actions.edit(details.id)}
          >
            {t.actions.edit}
          </GuardedButton>
        )}
        {connection && (
          <Button aria-expanded={commandShown} onClick={toggleCommand}>
            {commandShown ? t.actions.hideCommand : t.actions.showCommand}
          </Button>
        )}
        <Button loading={actions.busy.duplicate} onClick={() => actions.duplicate(details.id)}>
          {t.actions.duplicate}
        </Button>
        <Button
          variant="danger-text"
          loading={actions.busy.remove}
          onClick={() => actions.remove(details.id)}
        >
          {t.actions.delete}
        </Button>
      </div>
    </header>
  );
}
