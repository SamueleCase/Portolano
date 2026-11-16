import { useQuery } from "@tanstack/react-query";
import { useState } from "react";

import type { ChosenFile, ImportPreviewView_Serialize, ImportResult } from "../../bindings";
import { presentFailure } from "../../ipc/errors";
import { useChooseImportFile, useImportApply } from "../../ipc/mutations";
import { vaultStatusQuery } from "../../ipc/queries";
import type { FileReading } from "./FileStep";
import { useImportAnswers } from "./importAnswers";
import type { ImportSettings } from "./OptionsStep";
import { outcomeTotals } from "./reportSummary";
import { importOptions, useWizardPreview } from "./useWizardPreview";

/** The four steps of the wizard (inventario §5.13). */
export type WizardStep = "file" | "options" | "passwords" | "report";

const INITIAL_SETTINGS: ImportSettings = {
  into: null,
  onConflict: "fail",
  includeTrash: false,
  askPasswords: true,
};

/**
 * The reading of step 1: a dry run that cannot conflict with the document
 * ("Nuovi ID"), so that a file imported before still reads as readable;
 * the conflicts are for step 2.
 */
const READING = { into: null, onConflict: "newIds", includeTrash: false } as const;

/** The result of the import, with the passwords it asked for. */
export interface ImportOutcomeView {
  result: ImportResult;
  passwordsAsked: number;
}

/**
 * The flow of the import wizard: the file read at once, the dry runs of
 * step 2, the passwords one at a time, then a single `import_apply` with
 * every answer in the order of the requests (SPEC-UI §4.3). The typed
 * passwords wait in `answers` (a ref, see `ImportAnswers`), never in the
 * state below, and leave it on every path.
 */
export function useImportWizard() {
  const [step, setStep] = useState<WizardStep>("file");
  const [file, setFile] = useState<ChosenFile | null>(null);
  const [reading, setReading] = useState<FileReading | null>(null);
  const [trashSize, setTrashSize] = useState(0);
  const [settings, setSettings] = useState(INITIAL_SETTINGS);
  const [preview, setPreview] = useState<ImportPreviewView_Serialize | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [progress, setProgress] = useState({ position: 0, entered: 0, skipped: 0 });
  const [outcome, setOutcome] = useState<ImportOutcomeView | null>(null);
  const answers = useImportAnswers();
  const dryRun = useWizardPreview();
  const choose = useChooseImportFile();
  const apply = useImportApply();
  const vault = useQuery(vaultStatusQuery);

  const requests = preview?.requests ?? [];
  const asking = settings.askPasswords && requests.length > 0;
  const vaultLocked = vault.data?.locked ?? false;

  const chooseFile = async () => {
    let chosen: ChosenFile | null;
    try {
      chosen = await choose.mutateAsync();
    } catch (failure) {
      setReading({ state: "unreadable", message: presentFailure(failure).message });
      return;
    }
    if (chosen === null) return;
    setFile(chosen);
    setPreview(null);
    setReading({ state: "reading" });
    const answer = await dryRun.run(READING);
    if (answer === null) return;
    if (answer.status === "error") {
      setReading({ state: "unreadable", message: answer.message });
      return;
    }
    const totals = outcomeTotals(answer.preview.result);
    setReading({ state: "readable", objects: totals.read });
    setTrashSize(totals.skipped);
  };

  const refresh = async (next: ImportSettings) => {
    setPreview(null);
    const answer = await dryRun.run(importOptions(next));
    if (answer === null) return;
    setPreview(answer.status === "ok" ? answer.preview : null);
    setError(answer.status === "error" ? answer.message : null);
  };

  const changeSettings = (change: Partial<ImportSettings>) => {
    const next = { ...settings, ...change };
    setSettings(next);
    setError(null);
    if (Object.keys(change).some((key) => key !== "askPasswords")) void refresh(next);
  };

  const finish = async () => {
    const asked = requests.length;
    // The answers leave the ref here, whatever the import does next.
    const given = answers.current.take(requests);
    try {
      const result = await apply.mutateAsync({ options: importOptions(settings), answers: given });
      setOutcome({ result, passwordsAsked: asked });
      setStep("report");
    } catch (failure) {
      setStep("options");
      await refresh(settings);
      setError(presentFailure(failure).message);
    }
  };

  const next = () => {
    if (step === "file" && reading?.state === "readable") {
      setStep("options");
      void refresh(settings);
    } else if (step === "options" && preview !== null && !dryRun.previewing) {
      if (!asking) {
        void finish();
      } else if (!vaultLocked) {
        answers.current.start(requests);
        setProgress({ position: 0, entered: 0, skipped: 0 });
        setStep("passwords");
      }
    }
  };

  /** The answer to the current request: a password, or null (an empty field) to skip it. */
  const answer = (value: string | null) => {
    const held = answers.current;
    if (step !== "passwords" || held.complete) return;
    held.record(value === "" ? null : value);
    setProgress(({ position, entered, skipped }) =>
      value
        ? { position: position + 1, entered: entered + 1, skipped }
        : { position: position + 1, entered, skipped: skipped + 1 },
    );
    if (held.complete) void finish();
  };

  /** "Indietro" of step 2; step 3 has none, as in the mock: Esc closes the wizard. */
  const back = () => {
    if (step === "options") setStep("file");
  };

  return {
    step,
    file,
    reading,
    trashSize,
    settings,
    preview,
    error,
    progress,
    outcome,
    requests,
    asking,
    vaultLocked,
    previewing: dryRun.previewing,
    choosing: choose.isPending,
    applying: apply.isPending,
    chooseFile,
    changeSettings,
    next,
    answer,
    skipRemaining: () => void finish(),
    back,
  };
}
