import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { DetailPanel } from "./DetailPanel";
import { testJournal } from "./test-fixture";
vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
afterEach(() => {
  cleanup();
  vi.mocked(invoke).mockReset();
});
const panel = (profileId: string) => (
  <DetailPanel
    entry={testJournal.entries[0]}
    language="eng"
    native
    profileId={profileId}
    onClose={() => {}}
    onSource={() => {}}
    onEntry={() => {}}
  />
);
it("loads and saves notes against the displayed profile, not the active game profile", async () => {
  const notes: Record<string, string> = {
    "101": "Ari note",
    "202": "Amelia note",
  };
  vi.mocked(invoke).mockImplementation(async (command, args) => {
    const { profileId, text } = args as { profileId: string; text?: string };
    if (!profileId) throw new Error("Missing profile");
    if (command === "save_journal_note") notes[profileId] = text!;
    return notes[profileId];
  });
  const { rerender } = render(panel("101"));
  await waitFor(() =>
    expect(screen.getByRole("textbox")).toHaveValue("Ari note"),
  );
  rerender(panel("202"));
  await waitFor(() =>
    expect(screen.getByRole("textbox")).toHaveValue("Amelia note"),
  );
  fireEvent.change(screen.getByRole("textbox"), {
    target: { value: "Amelia updated" },
  });
  fireEvent.click(screen.getByRole("button", { name: "Save note" }));
  await waitFor(() => expect(screen.getByText("Saved locally")).toBeVisible());
  expect(notes).toEqual({ "101": "Ari note", "202": "Amelia updated" });
});
it("does not allow blank replacement when an existing note cannot be read", async () => {
  vi.mocked(invoke).mockRejectedValue(new Error("Unreadable note"));
  render(panel("101"));
  await screen.findByRole("status");
  expect(screen.getByRole("textbox")).toBeDisabled();
  expect(screen.getByRole("button", { name: "Save note" })).toBeDisabled();
});
