import { useQuery } from "@tanstack/react-query";
import type { ReactNode } from "react";

import { Button } from "../../components/Button";
import { Dialog } from "../../components/Dialog";
import { GroupHeader } from "../../components/GroupHeader";
import type { Segment } from "../../components/SegmentedControl";
import { texts } from "../../i18n/it";
import { appStatusQuery } from "../../ipc/queries";
import { AppearanceSettings } from "./AppearanceSettings";
import { ChoiceSetting } from "./ChoiceSetting";
import { PathSetting } from "./PathSetting";
import styles from "./Preferences.module.css";
import { TerminalCommandSetting } from "./TerminalCommandSetting";
import { useConfigSettings } from "./useConfigSettings";

interface PreferencesDialogProps {
  open: boolean;
  onClose: () => void;
}

/** The values of the choices of `config.toml` (SPEC §9), in the order shown. */
const BACKENDS = ["keychain", "file"] as const;
const TERMINALS = ["terminal", "iterm", "ghostty", "custom"] as const;
const CLIENTS = ["auto", "freerdp", "windows-app"] as const;
const CERTS = ["tofu", "ignore", "deny"] as const;

/** The segments of `values`, labelled by `labels`. */
function segments<T extends string>(
  values: readonly T[],
  labels: Readonly<Record<T, string>>,
): Segment<T>[] {
  return values.map((value) => ({ value, label: labels[value] }));
}

/** `value` if it is one of `values`, else `fallback` (the default of SPEC §9). */
function oneOf<T extends string>(
  value: string | undefined,
  values: readonly T[],
  fallback: NoInfer<T>,
): T {
  return values.find((option) => option === value) ?? fallback;
}

function Group({ title, children }: { title: string; children: ReactNode }) {
  return (
    <>
      <div className={styles.groupTitle}>
        <GroupHeader>{title}</GroupHeader>
      </div>
      <div className={styles.groupBody}>{children}</div>
    </>
  );
}

/**
 * The preferences (inventario §5.16): the keys of `config.toml` through
 * `config_set`/`config_reset` and the native file dialogs, the appearance
 * through `ui_prefs_set`. Everything applies at once; the fields that
 * depend on a choice appear only with it.
 */
export function PreferencesDialog({ open, onClose }: PreferencesDialogProps) {
  const t = texts.preferences;
  const settings = useConfigSettings();
  const { data: status } = useQuery({ ...appStatusQuery, enabled: open });
  const backend = oneOf(settings.value("vault.backend"), BACKENDS, "keychain");
  const terminal = oneOf(settings.value("terminal.app"), TERMINALS, "terminal");

  return (
    <Dialog
      open={open}
      onClose={onClose}
      title={t.title}
      width="preferences"
      footer={
        <Button variant="primary" onClick={onClose}>
          {t.close}
        </Button>
      }
    >
      <p className={styles.hint}>{t.hint}</p>
      <div className={styles.grid}>
        <Group title={t.groups.document}>
          <PathSetting
            settings={settings}
            configKey="default_document"
            label={t.document.label}
            fallback={status?.defaultDocument.path ?? ""}
          />
        </Group>
        <Group title={t.groups.vault}>
          <ChoiceSetting
            label={t.vault.label}
            options={segments(BACKENDS, t.vault.backends)}
            value={backend}
            onChange={(value) => settings.set("vault.backend", value)}
            error={settings.error("vault.backend")}
          />
          {backend === "file" && (
            <PathSetting
              settings={settings}
              configKey="vault.file"
              label={t.vault.file}
              fallback={t.vault.fileDefault}
            />
          )}
        </Group>
        <Group title={t.groups.terminal}>
          <ChoiceSetting
            label={t.terminal.label}
            options={segments(TERMINALS, t.terminal.apps)}
            value={terminal}
            onChange={(value) => settings.set("terminal.app", value)}
            error={settings.error("terminal.app")}
          />
          {terminal === "custom" && <TerminalCommandSetting settings={settings} />}
        </Group>
        <Group title={t.groups.ssh}>
          <PathSetting
            settings={settings}
            configKey="ssh.program"
            label={t.ssh.label}
            fallback={t.ssh.default}
            hint={t.ssh.hint}
          />
        </Group>
        <Group title={t.groups.rdp}>
          <ChoiceSetting
            label={t.rdp.client}
            options={segments(CLIENTS, t.rdp.clients)}
            value={oneOf(settings.value("rdp.client"), CLIENTS, "auto")}
            onChange={(value) => settings.set("rdp.client", value)}
            error={settings.error("rdp.client")}
          />
          <PathSetting
            settings={settings}
            configKey="rdp.freerdp_program"
            label={t.rdp.freerdp}
            fallback={t.rdp.freerdpDefault}
            hint={t.ssh.hint}
          />
          <ChoiceSetting
            label={t.rdp.cert}
            options={segments(CERTS, t.rdp.certs)}
            value={oneOf(settings.value("rdp.cert"), CERTS, "tofu")}
            onChange={(value) => settings.set("rdp.cert", value)}
            error={settings.error("rdp.cert")}
          />
        </Group>
        <Group title={t.groups.appearance}>
          <AppearanceSettings enabled={open} />
        </Group>
      </div>
    </Dialog>
  );
}
