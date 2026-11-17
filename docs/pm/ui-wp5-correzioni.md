# UI-WP5 – Correzioni della revisione completa (2026-11-17)

Revisione indipendente in due parti (Rust e frontend) del branch `worktree-agent-aae11fcea64bbed01` a `a0d3a32`.
Gate rieseguiti dal PM su `a0d3a32`: tutti verdi (Vitest 710, e2e 82/82 due volte, smoke OK). Decisioni del PM in
DEC-043.

## Rust
1. **(importante)** `crates/portolano-ops/src/nodes/tests/staged_move.rs`: test dello spostamento con segreti in
   staging e salvataggio fallito (`modify_externally()`): il nodo non si sposta, la credenziale e il vault restano
   invariati (modello: `staged::failed_save_keeps_the_old_password_and_writes_nothing`); e un test di successo con
   spostamento più segreto (`path`, `secrets_written`).
2. `dto/inputs/edit.rs`: `ParentInput.id` con la chiave obbligatoria anche se `null`
   (`#[serde(deserialize_with = "Option::deserialize")]` o equivalente), bindings rigenerati (`parent?: { id: string |
   null }`), test che `{ "parent": {} }` è rifiutato.
3. `nodes/edit.rs`: uno spostamento impossibile (cartella in un proprio discendente, destinazione non cartella,
   profondità) dà l'errore dello spostamento e non `NameTaken`: lo spostamento sulla copia prima del controllo del
   nome finale (DEC-043.5).
4. `nodes/edit.rs`: `check_edit` riceve il padre attuale già calcolato invece di ricalcolarlo.
5. `nodes/edit.rs`: la sezione `# Errors` cita `NodeNotFound` per una cartella di destinazione inesistente.
6. `names(doc, parent)` dei test in `testutil` del crate, condivisa con `moves.rs`.
7. Il test della cartella spostata in un discendente: nome `a_folder_cannot_move_into_its_descendant`, `matches!`
   sull'errore `InvalidMove`, stato controllato anche su `stored.doc`.
8. Test mancanti: `parent` uguale al padre attuale (nessun riordino), destinazione non cartella (`NotAFolder`),
   superamento di `MAX_DEPTH`.
9. Un comportamento per test: dividere `parent_moves_to_a_folder_or_to_the_root_and_its_absence_stays`
   (`dto/inputs/tests.rs`) e `refused_ssh_option_names_the_setting_and_the_option` (`error/tests.rs`: campo e testo
   in due test).
10. `error.rs` `settings_field`: test per `settings` da solo e per `settings.<chiave>` di una chiave non ssh (es. rdp);
    nei connettori un test che verifica `setting` per una chiave sconosciuta.

## Frontend
- **B1 (bloccante)** Id duplicati delle righe nuove: i contatori `useRef(0)` in `PropertiesEditor.tsx`, `SshStep.tsx`,
  `JumpHostList.tsx` ripartono da zero quando il passo si smonta (Avanti/Indietro), quindi una seconda riga riprende
  `new:1` (chiavi React duplicate, nomi che cambiano insieme, valore in staging associato alla riga sbagliata). Id
  generati in un punto che sopravvive ai passi (contatore dell'editor nel contesto, o `crypto.randomUUID()`), con test:
  riga aggiunta → Avanti → Indietro → seconda riga, per proprietà segrete, opzioni ssh e jump host.
- **I1** Vault bloccato: nell'editor un segreto già salvato deve potersi sostituire e cancellare in staging (lo sblocco
  arriva al Salva). `secretSlots.ts:62-63` + `SecretRow` disattivano il bottone e mostrano "Imposta…": tenere le azioni
  attive con il vault bloccato (stato o prop separati dal badge, senza modificare `SecretRow` se possibile; se serve una
  prop nuova in `components/SecretRow.tsx`, è concessa come eccezione DEC-043.6), `replacing` calcolato da
  `secrets.stored(key)` (titolo "Sostituisci…" corretto), test: segreto salvato, vault bloccato, "Sostituisci…", Salva
  con sblocco.
- **I2** Focus: a ogni cambio di passo sul primo campo del passo (o sull'intestazione), al montaggio della forma dopo
  il caricamento su "Nome", dopo un errore del backend sul campo indicato (anche quando cambia passo). Test.
- **I3** Duplicati della cornice: `features/dialogs/nodePaths.ts`, `treeIndex.ts` (`nodeKindOf`, `indexTree`),
  `iconChoice.ts` (`iconFileName`), `JumpHostList.tsx` (`hostOf`, senza IPv6) ricopiano helper di
  `features/tree/treeModel.ts` (`pathSegments`, `readablePath`, `pathName`, `iconChoice`, `summaryHost`, …): usare
  quelli (DEC-043.4). Dentro il WP: `same()` in `editorRequest.ts` e `externalChanges.ts`; `objectOf`/`JsonObject` in
  `valuesOf.ts` e `settingsOf.ts`: una sola copia.
- **m1** `useExternalChanges.ts:48`: l'avviso "documento modificato" resta dopo "Ricarica" se il nodo aperto non è
  cambiato (structural sharing): chiuderlo quando la query del dettaglio torna valida dopo l'evento (es.
  `dataUpdatedAt`). Test.
- **m2** `serverErrors.ts`: il campo `property:<nome>` del backend va sotto le proprietà (passo 1), non su `form`. Test.
- **m3** `NodeEditor.tsx:58`: durante il caricamento in modifica il pannello non si chiama "Nuovo elemento".
- **m4** `NodeEditor.test.tsx`: `expectNoSentinel` controlla anche gli store Zustand (`useRequests`, `useToasts`,
  `useWorkspace`, sessioni).
- **m5** Test Vitest di `LoginDialog`: campi svuotati dopo la consegna, dominio solo con `ask_domain`, Annulla → `null`.
- **m6** Messaggi d'errore di gruppo collegati ai controlli con `aria-describedby` (`GeneralStep.tsx` icona e colore,
  `PositionField.tsx`, `PropertiesEditor.tsx`, `JumpHostList.tsx`, `SshStep.tsx`, `CredentialSetting.tsx`,
  `preferences/ChoiceSetting.tsx`).
- **m8** `features/dialogs/renderInWindow.tsx` (utilità di test con `afterEach` di vitest) si sposta sotto
  `frontend/src/test/` (DEC-043.2).
- m7 (`checkSettings` nel finto) e m9 ("Avanti" secondario): approvati così (DEC-043.1, DEC-043.3), nessuna modifica.
