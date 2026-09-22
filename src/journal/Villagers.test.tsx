import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { Villagers } from "./screens";
import { testJournal } from "./test-fixture";
import { defaultFilters, type Hint } from "./types";

afterEach(cleanup);

it("keeps the reaction of an untried gift hidden while opening its hint", () => {
  const hint: Hint = { kind: "gift", activity: "dishes", areas: [], seasons: [], source: null };
  const journal = {
    ...testJournal,
    villagers: [{
      ...testJournal.villagers[0],
      untried: [{ key: "g0:untried:0", entry: null, name: null, art: null, found: false, revealed: false, hint }],
    }],
  };
  const onHint = vi.fn();
  render(<Villagers journal={journal} language="eng" filters={defaultFilters} onEntry={() => {}} onHint={onHint} onNavigate={() => {}} />);
  expect(screen.getByText("Untried gifts")).toBeVisible();
  expect(screen.queryByText("Loved")).not.toBeInTheDocument();
  expect(screen.queryByText("Liked")).not.toBeInTheDocument();
  fireEvent.click(screen.getByRole("button", { name: "Undiscovered" }));
  expect(onHint).toHaveBeenCalledWith(hint);
});
