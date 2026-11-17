import type { FakeKind } from "./fakeBackendModel";

/**
 * The Royal TSX document that the import of the fake backend reads: invented
 * data only (SPEC §1.3), with a node for each case of the wizard.
 */

/** An object of the fake Royal document. */
export interface SourceObject {
  sourceId: string;
  sourceType: string;
  /** The name of its node. */
  name: string;
  /** Its name in Royal, when the node has another one: the trash, "Cestino (Royal)". */
  royalName?: string;
  kind: FakeKind;
  /** The object that contains it; none for the document itself. */
  parent: string | null;
  protocol?: string;
  host?: string;
  username?: string;
  /** Its encrypted password, asked in step 3 of the wizard. */
  asksPassword?: boolean;
  /** Why it is partial or unsupported, whatever the answers. */
  reasons?: string[];
  /** In the trash of Royal TSX. */
  inTrash?: boolean;
}

const royalId = (serial: number) =>
  `00000000-0000-4000-8000-${(0xa0000 + serial).toString(16).padStart(12, "0")}`;

export const DOCUMENT = royalId(0);
const PRODUCTION = royalId(1);
const TRASH = royalId(7);

export const OBJECTS: readonly SourceObject[] = [
  {
    sourceId: DOCUMENT,
    sourceType: "RoyalDocument",
    name: "Ufficio",
    kind: "folder",
    parent: null,
  },
  {
    sourceId: PRODUCTION,
    sourceType: "RoyalFolder",
    name: "Produzione",
    kind: "folder",
    parent: DOCUMENT,
  },
  {
    sourceId: royalId(2),
    sourceType: "RoyalSSHConnection",
    name: "web10",
    kind: "connection",
    parent: PRODUCTION,
    protocol: "ssh",
    host: "198.51.100.10",
    username: "deploy",
    asksPassword: true,
  },
  {
    sourceId: royalId(3),
    sourceType: "RoyalSSHConnection",
    name: "db10",
    kind: "connection",
    parent: PRODUCTION,
    protocol: "ssh",
    host: "db10.example.com",
    username: "dba",
    asksPassword: true,
  },
  {
    sourceId: royalId(4),
    sourceType: "RoyalRDSConnection",
    name: "dc10",
    kind: "connection",
    parent: PRODUCTION,
    protocol: "rdp",
    host: "198.51.100.20",
    reasons: ["icona personalizzata oltre 4 MiB: conservata in extra"],
  },
  {
    sourceId: royalId(5),
    sourceType: "RoyalCredential",
    name: "Admin ufficio",
    kind: "credential",
    parent: DOCUMENT,
    username: "admin",
    asksPassword: true,
  },
  {
    sourceId: royalId(6),
    sourceType: "RoyalVNCConnection",
    name: "vnc-ufficio",
    kind: "unsupported",
    parent: DOCUMENT,
    reasons: ["tipo Royal non supportato: RoyalVNCConnection"],
  },
  {
    sourceId: TRASH,
    sourceType: "RoyalTrash",
    name: "Cestino (Royal)",
    royalName: "Cestino",
    kind: "folder",
    parent: DOCUMENT,
    inTrash: true,
  },
  {
    sourceId: royalId(8),
    sourceType: "RoyalSSHConnection",
    name: "old-web",
    kind: "connection",
    parent: TRASH,
    protocol: "ssh",
    host: "192.0.2.50",
    inTrash: true,
  },
];

/**
 * The path of `object` in the report and the requests. As in the importer, a
 * skipped trash keeps its Royal name; an imported one is "Cestino (Royal)".
 */
export function pathOfObject(object: SourceObject, includeTrash = false): string {
  const parent = OBJECTS.find((candidate) => candidate.sourceId === object.parent);
  const name = object.inTrash && !includeTrash ? (object.royalName ?? object.name) : object.name;
  return parent ? `${pathOfObject(parent, includeTrash)}/${name}` : name;
}
