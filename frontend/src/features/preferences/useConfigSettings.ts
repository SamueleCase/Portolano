import { useQuery } from "@tanstack/react-query";
import { useState } from "react";

import { presentFailure } from "../../ipc/errors";
import { useChooseConfigFile, useConfigReset, useConfigSet } from "../../ipc/mutations";
import { configQuery } from "../../ipc/queries";

/** The keys of `config.toml` the preferences change (SPEC §9). */
export type ConfigKey =
  | "default_document"
  | "vault.backend"
  | "vault.file"
  | "terminal.app"
  | "terminal.command"
  | "ssh.program"
  | "rdp.client"
  | "rdp.freerdp_program"
  | "rdp.cert";

/** The configuration as the preferences read and change it. */
export interface ConfigSettings {
  /** The value set in `config.toml`, if any: the preferences show the default otherwise. */
  value: (key: ConfigKey) => string | undefined;
  set: (key: ConfigKey, value: string) => void;
  /** Back to the default (`config_reset`). */
  reset: (key: ConfigKey) => void;
  /** A file chosen with the native dialog of Rust (`choose_config_file`). */
  choose: (key: ConfigKey) => void;
  /** The message of the backend for the last change of `key`, if it was refused. */
  error: (key: ConfigKey) => string | undefined;
}

/**
 * The keys of the configuration with their changes, applied at once and
 * validated by the backend (DEC-008, DEC-011): a refused change leaves its
 * message under its control.
 */
export function useConfigSettings(): ConfigSettings {
  const { data: entries = [] } = useQuery(configQuery);
  const setValue = useConfigSet();
  const resetValue = useConfigReset();
  const chooseFile = useChooseConfigFile();
  const [errors, setErrors] = useState<Partial<Record<ConfigKey, string>>>({});

  const run = (key: ConfigKey, change: Promise<unknown>) => {
    setErrors((current) => ({ ...current, [key]: undefined }));
    change.catch((failure: unknown) =>
      setErrors((current) => ({ ...current, [key]: presentFailure(failure).message })),
    );
  };

  return {
    value: (key) => entries.find((entry) => entry.key === key)?.value,
    set: (key, value) => run(key, setValue.mutateAsync({ key, value })),
    reset: (key) => run(key, resetValue.mutateAsync(key)),
    choose: (key) => run(key, chooseFile.mutateAsync(key)),
    error: (key) => errors[key],
  };
}
