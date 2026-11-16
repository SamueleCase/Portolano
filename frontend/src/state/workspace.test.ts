import { beforeEach, describe, expect, it } from "vitest";

import { DETAIL_TAB, useWorkspace, VALIDATION_TAB } from "./workspace";

const workspace = () => useWorkspace.getState();

describe("the workspace store", () => {
  beforeEach(() => {
    workspace().documentSwitched();
    useWorkspace.setState({ searchRequests: 0, connectionMenuOpen: false });
  });

  it("shows the detail of a node it selects, with the command hidden", () => {
    workspace().showTab("fake-session-1");
    workspace().toggleCommand();

    workspace().select("node-a");

    expect(workspace()).toMatchObject({
      selectedId: "node-a",
      activeTab: DETAIL_TAB,
      commandShown: false,
    });
  });

  it("brings the detail in front when the tab in front closes", () => {
    workspace().openValidation();

    workspace().tabClosed(VALIDATION_TAB);

    expect(workspace()).toMatchObject({ activeTab: DETAIL_TAB, validationOpen: false });
  });

  it("keeps the tab in front when another tab closes", () => {
    workspace().showTab("fake-session-2");

    workspace().tabClosed("fake-session-1");

    expect(workspace().activeTab).toBe("fake-session-2");
  });

  it("keeps a session replaced by «Riprova» in front", () => {
    workspace().showTab("fake-session-1");

    workspace().tabReplaced("fake-session-1", "fake-session-2");

    expect(workspace().activeTab).toBe("fake-session-2");
  });

  it("opens the folders of a node it reveals, keeping those already open", () => {
    workspace().setExpanded(["folder-a"]);

    workspace().reveal("node-b", ["folder-b", "folder-a"]);

    expect(workspace().selectedId).toBe("node-b");
    expect(workspace().expanded).toEqual(["folder-a", "folder-b"]);
  });

  it("forgets the selection, the search and the banner of the previous document", () => {
    workspace().select("node-a");
    workspace().setQuery("web");
    workspace().setBanner({ kind: "missing", fileName: "portolano.json" });

    workspace().documentSwitched();

    expect(workspace()).toMatchObject({
      selectedId: null,
      query: "",
      banner: null,
      expanded: null,
    });
  });

  it("forgets the summary of a session that is no longer open", () => {
    workspace().sessionOpened("fake-session-1", "deploy@bastion.example.com:22");
    workspace().sessionOpened("fake-session-2", "admin@web01.example.com:2222");

    workspace().sessionsClosed(["fake-session-2"]);

    expect(workspace().sessionSummaries).toEqual({
      "fake-session-2": "admin@web01.example.com:2222",
    });
  });
});
