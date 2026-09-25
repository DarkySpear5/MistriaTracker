import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { ArtworkProvider } from "./Artwork";
import { Villagers } from "./screens";
import { testJournal } from "./test-fixture";
import { defaultFilters, type Hint } from "./types";

afterEach(cleanup);

it("shows a shadowed gift shape only when gentle hints are enabled", async () => {
  const hint: Hint = { kind: "gift", activity: "dishes", areas: [], seasons: [], source: null };
  const journal = {
    ...testJournal,
    villagers: [{
      ...testJournal.villagers[0],
      untried: [{ key: "g0:untried:0", entry: null, name: null, art: "g0:untried:0", found: false, revealed: false, hint }],
    }],
  };
  const onHint = vi.fn();
  const artLoader = vi.fn(async (keys: string[]) =>
    Object.fromEntries(keys.map((key) => [key, "data:image/png;base64,AA=="])),
  );
  const view = render(
    <ArtworkProvider loader={artLoader} scope="test">
      <Villagers
        journal={journal}
        language="eng"
        filters={defaultFilters}
        hintsEnabled={false}
        onEntry={() => {}}
        onHint={onHint}
        onNavigate={() => {}}
      />
    </ArtworkProvider>,
  );
  expect(screen.getByText("Untried gifts")).toBeVisible();
  expect(screen.queryByText("Loved")).not.toBeInTheDocument();
  expect(screen.queryByText("Liked")).not.toBeInTheDocument();
  await act(async () => {
    await Promise.resolve();
  });
  expect(artLoader).not.toHaveBeenCalled();
  view.rerender(
    <ArtworkProvider loader={artLoader} scope="hints-enabled">
      <Villagers
        journal={journal}
        language="eng"
        filters={defaultFilters}
        hintsEnabled
        onEntry={() => {}}
        onHint={onHint}
        onNavigate={() => {}}
      />
    </ArtworkProvider>,
  );
  await waitFor(() => expect(artLoader).toHaveBeenCalledWith(["g0:untried:0"]));
  fireEvent.click(screen.getByRole("button", { name: "Undiscovered" }));
  expect(onHint).toHaveBeenCalledWith(hint);
});
