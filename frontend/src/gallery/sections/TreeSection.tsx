import { TreeRow } from "../../components/TreeRow";
import { gallery } from "../../i18n/gallery";
import { hostileName } from "../fixtures";
import styles from "../Gallery.module.css";
import { GallerySection } from "../GallerySection";

/**
 * Every state of a tree row (inventario §4.19). The real tree, with Headless
 * Tree, is UI-WP4's: here the rows sit in a plain `tree`.
 */
export function TreeSection() {
  return (
    <GallerySection id="tree" title={gallery.section.tree}>
      <div role="tree" aria-label={gallery.section.tree} className={styles.column}>
        <TreeRow
          name="Produzione"
          kind="folder"
          depth={0}
          expanded
          meta={{ kind: "descendants", count: 6 }}
        />
        <TreeRow
          name="web01"
          kind="ssh"
          depth={1}
          color="#7FC2A0"
          meta={{ kind: "host", host: "web01.example.com" }}
          session="connecting"
        />
        <TreeRow
          name="bastion"
          kind="ssh"
          depth={1}
          selected
          favorite
          meta={{ kind: "host", host: "bastion.example.com" }}
          session="connected"
        />
        <TreeRow
          name="db02"
          kind="ssh"
          depth={1}
          meta={{ kind: "host", host: "192.0.2.15" }}
          session="failed"
        />
        <TreeRow
          name="win-app01"
          kind="rdp"
          depth={1}
          meta={{ kind: "host", host: "198.51.100.7" }}
          dragging
        />
        <TreeRow
          name="db01"
          kind="ssh"
          depth={1}
          icon={{ source: "builtin", name: "database" }}
          meta={{ kind: "host", host: "db01.example.com" }}
          dropTarget="before"
        />
        <TreeRow
          name="Staging"
          kind="folder"
          depth={0}
          meta={{ kind: "descendants", count: 2 }}
          dropTarget="inside"
        />
        <TreeRow name="Credenziali" kind="folder" depth={0} expanded />
        <TreeRow
          name="deploy"
          kind="credential"
          depth={1}
          meta={{ kind: "secret", state: "present", backend: "keychain" }}
        />
        <TreeRow
          name="admin · ACME"
          kind="credential"
          depth={1}
          meta={{ kind: "secret", state: "absent", backend: "keychain" }}
        />
        <TreeRow
          name="ops"
          kind="credential"
          depth={1}
          meta={{ kind: "secret", state: "locked", backend: "file" }}
        />
        <TreeRow name="vnc-legacy" kind="unsupported" depth={0} />
        <TreeRow
          name="bastion-staging"
          kind="ssh"
          depth={0}
          highlight="bast"
          meta={{ kind: "host", host: "192.0.2.30" }}
        />
        <TreeRow
          name={hostileName}
          kind="ssh"
          depth={0}
          meta={{ kind: "host", host: "192.0.2.66\u0007" }}
        />
      </div>
    </GallerySection>
  );
}
