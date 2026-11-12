import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { useState } from "react";
import { describe, expect, it } from "vitest";

import { FavoriteToggle } from "./FavoriteToggle";
import { TagInput } from "./TagInput";

function Tags() {
  const [tags, setTags] = useState(["web"]);
  return <TagInput label="Tag" tags={tags} onChange={setTags} />;
}

describe("TagInput", () => {
  it("adds the written tag with Enter", async () => {
    render(<Tags />);

    await userEvent.type(screen.getByRole("textbox", { name: "Tag" }), "postgres{Enter}");

    expect(screen.getByText("postgres")).toBeInTheDocument();
    expect(screen.getByRole("textbox", { name: "Tag" })).toHaveValue("");
  });

  it("does not add a tag twice", async () => {
    render(<Tags />);

    await userEvent.type(screen.getByRole("textbox", { name: "Tag" }), "web,");

    expect(screen.getAllByText("web")).toHaveLength(1);
  });

  it("removes the last tag with Backspace on the empty field", async () => {
    render(<Tags />);

    await userEvent.type(screen.getByRole("textbox", { name: "Tag" }), "{Backspace}");

    expect(screen.queryByText("web")).not.toBeInTheDocument();
  });
});

describe("FavoriteToggle", () => {
  it("announces whether the node is a favorite", async () => {
    function Favorite() {
      const [favorite, setFavorite] = useState(false);
      return <FavoriteToggle favorite={favorite} onChange={setFavorite} />;
    }
    render(<Favorite />);
    const toggle = screen.getByRole("button", { name: "Preferito" });

    await userEvent.click(toggle);

    expect(toggle).toHaveAttribute("aria-pressed", "true");
  });
});
