import type {
  ImportPreviewView_Serialize,
  ImportResult,
  Issue_Serialize,
  SecretRequestView_Serialize,
} from "../../bindings";

/**
 * The fixed data of the gallery section of the import and the validation:
 * an invented Royal TSX document (SPEC §1.3), never a real one.
 */

const counts = (source: number, imported: number, partial = 0, unsupported = 0, skipped = 0) => ({
  source,
  imported,
  partial,
  unsupported,
  skipped,
});

/** The request step 3 shows in the gallery: the second of four. */
export const galleryRequest: SecretRequestView_Serialize = {
  nodePath: "Sede/Produzione/db20",
  username: "dba",
  field: { kind: "password" },
};

export const galleryRequests: SecretRequestView_Serialize[] = [
  { nodePath: "Sede/Produzione/web20", username: "deploy", field: { kind: "password" } },
  galleryRequest,
  { nodePath: "Sede/Admin sede", username: "admin", field: { kind: "password" } },
  { nodePath: "Sede/Rete/bastion20", username: null, field: { kind: "passphrase" } },
];

export const galleryResult: ImportResult = {
  report: {
    importer: "royal",
    source_file: "sede.rtsz",
    document_name: "Sede",
    imported_at: "2026-11-10T10:00:00Z",
    counts: {
      RoyalDocument: counts(1, 1),
      RoyalFolder: counts(3, 3),
      RoyalSSHConnection: counts(6, 3, 2, 0, 1),
      RoyalRDSConnection: counts(3, 2, 1),
      RoyalCredential: counts(2, 1, 1),
      RoyalVNCConnection: counts(1, 0, 0, 1),
      RoyalTrash: counts(1, 0, 0, 0, 1),
    },
    items: [
      {
        source_type: "RoyalSSHConnection",
        source_id: "royal-web20",
        path: "Sede/Produzione/web20",
        status: "partial",
        reasons: ["password Royal cifrata non importata: impostala dal nodo"],
      },
      {
        source_type: "RoyalRDSConnection",
        source_id: "royal-ts20",
        path: "Sede/Produzione/ts20",
        status: "partial",
        reasons: ["icona personalizzata oltre 4 MiB: conservata in extra"],
      },
      {
        source_type: "RoyalSSHConnection",
        source_id: "royal-db20",
        path: "Sede/Produzione/db20",
        status: "partial",
        reasons: ["password Royal cifrata non importata: impostala dal nodo"],
      },
      {
        source_type: "RoyalCredential",
        source_id: "royal-admin",
        path: "Sede/Admin sede",
        status: "partial",
        reasons: ["password Royal cifrata non importata: impostala dal nodo"],
      },
      {
        source_type: "RoyalVNCConnection",
        source_id: "royal-vnc20",
        path: "Sede/vnc-sede",
        status: "unsupported",
        reasons: ["tipo Royal non supportato: RoyalVNCConnection"],
      },
      {
        source_type: "RoyalTrash",
        source_id: "royal-trash",
        path: "Sede/Cestino",
        status: "skipped",
        reasons: ["cestino di Royal non importato (usa l'opzione per includerlo)"],
      },
      {
        source_type: "RoyalSSHConnection",
        source_id: "royal-old20",
        path: "Sede/Cestino/old20",
        status: "skipped",
        reasons: ["nel cestino di Royal"],
      },
    ],
    unmapped_fields: {},
    warnings: ["documento protetto da password"],
    kinds: { folders: 5, connections: { rdp: 3, ssh: 6 }, credentials: 2, unsupported: 1 },
    trash_items: 1,
  },
  root: "00000000-0000-4000-8000-0000000b0000",
  inserted: 15,
  updated: 0,
  secrets_written: 1,
  icons_stored: 0,
  warnings: [],
  cleanup_failures: [],
  dry_run: false,
};

export const galleryPreview: ImportPreviewView_Serialize = {
  result: { ...galleryResult, secrets_written: 0, dry_run: true },
  requests: galleryRequests,
};

const unsupportedIssue: Issue_Serialize = {
  severity: "info",
  node: "node-vnc",
  code: "unsupported_node",
  message: "il nodo 'vnc-sede' non è supportato",
};

export const galleryIssues: Issue_Serialize[] = [
  {
    severity: "warning",
    node: "node-db20",
    code: "missing_secret",
    message: "un segreto del nodo 'db20' non è nel vault",
  },
  {
    severity: "error",
    node: "node-old-gw",
    code: "dangling_ref",
    message: "il jump host fa riferimento a un nodo che non esiste più",
  },
  unsupportedIssue,
];

export const galleryLockedIssues: Issue_Serialize[] = [
  unsupportedIssue,
  {
    severity: "info",
    node: null,
    code: "vault_locked",
    message: "il vault è bloccato: la presenza dei segreti non è stata controllata",
  },
];

/** The readable paths of the nodes of the issues. */
export const galleryPaths: Readonly<Record<string, string>> = {
  "node-db20": "Sede / Produzione / db20",
  "node-old-gw": "Archivio / old-gw",
  "node-vnc": "Sede / vnc-sede",
};
