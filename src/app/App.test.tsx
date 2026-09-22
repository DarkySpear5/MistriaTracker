import {
  act,
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { App } from "./App";
import { testJournal } from "../journal/test-fixture";
vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/api/path", () => ({
  join: vi.fn().mockResolvedValue("C:/Test/mod_data"),
  localDataDir: vi.fn().mockResolvedValue("C:/Test"),
}));
const preview = () =>
  render(
    <App
      initialJournal={testJournal}
      nativeRuntime={() => false}
      artLoader={async () => ({})}
    />,
  );
const nav = (name: string) =>
  fireEvent.click(
    within(screen.getByRole("navigation")).getByRole("button", {
      name: new RegExp(name),
    }),
  );
const search = (query: string) =>
  fireEvent.change(screen.getByRole("combobox", { name: /search/i }), {
    target: { value: query },
  });
describe("Desktop journal", () => {
  it("shows that no save is loaded after the companion deactivates the current profile", async () => {
    let active: string | null = "ari";
    vi.useFakeTimers();
    vi.mocked(invoke).mockImplementation(async (command) => {
      if (command === "get_preferences")
        return { language_preference: "auto", effective_language: "eng", spoiler_mode: "free", hints_enabled: false };
      if (command === "get_profile_summary")
        return { active_profile: active, profiles: ["ari"] };
      if (command === "get_journal_snapshot") return active ? testJournal : null;
      if (command === "start_live_tracking") return;
      if (command === "poll_live_tracking") {
        active = null;
        return 1;
      }
      throw new Error("Unexpected command " + command);
    });
    try {
      render(
        <App
          nativeRuntime={() => true}
          resolveGameDirectory={async () => "C:/Test/Fields of Mistria"}
          readinessProbe={async () => ({
            catalog: { game_version: "verified", item_count: 3 },
            catalog_approved: true,
            companion_log_found: true,
          })}
        />,
      );
      await act(async () => {
        await vi.advanceTimersByTimeAsync(1);
      });
      expect(screen.getAllByText("Ari").length).toBeGreaterThan(0);

      await act(async () => {
        await vi.advanceTimersByTimeAsync(1500);
      });

      expect(screen.getByRole("heading", { name: "No save is currently loaded" })).toBeVisible();
      expect(screen.queryByText("Ari")).not.toBeInTheDocument();
    } finally {
      vi.useRealTimers();
    }
  });
  it("recovers polling status and notices a profile change without item events", async () => {
    let offline = false;
    let active = "ari";
    let starts = 0;
    vi.useFakeTimers();
    vi.mocked(invoke).mockImplementation(async (command) => {
      if (command === "get_preferences")
        return { language_preference: "auto", effective_language: "eng", spoiler_mode: "free", hints_enabled: false };
      if (command === "get_profile_summary")
        return { active_profile: active, profiles: ["ari", "amelia"] };
      if (command === "get_journal_snapshot")
        return {
          ...testJournal,
          profile: {
            id: active,
            name: active === "ari" ? "Ari" : "Amelia",
            farm: "Test Farm",
          },
        };
      if (command === "poll_live_tracking") {
        if (offline) throw new Error("Temporary log access failure");
        return 0;
      }
      if (command === "start_live_tracking") {
        if (++starts > 1) throw new Error("Must not throw away unread cursor");
        return;
      }
      throw new Error("Unexpected command " + command);
    });
    try {
      render(
        <App
          nativeRuntime={() => true}
          resolveGameDirectory={async () => "C:/Test/Fields of Mistria"}
          readinessProbe={async () => ({
            catalog: { game_version: "verified", item_count: 3 },
            catalog_approved: true,
            companion_log_found: true,
          })}
          importExisting={async () => ({
            profile_id: "ari",
            discovered_items: 2,
            imported: false,
          })}
        />,
      );
      await act(async () => {
        await vi.advanceTimersByTimeAsync(1);
      });
      fireEvent.click(screen.getByRole("button", { name: "Settings" }));
      offline = true;
      await act(async () => {
        await vi.advanceTimersByTimeAsync(1500);
      });
      expect(screen.getByTitle("Tracking unavailable")).toBeVisible();
      offline = false;
      active = "amelia";
      await act(async () => {
        await vi.advanceTimersByTimeAsync(1500);
      });
      expect(screen.getByTitle("Live tracking ready")).toBeVisible();
      expect(screen.getByText("Amelia")).toBeVisible();
    } finally {
      vi.useRealTimers();
    }
  });
  it("connects automatically when the companion appears after startup", async () => {
    let companion = false;
    vi.useFakeTimers();
    vi.mocked(invoke).mockImplementation(async (command) => {
      if (command === "get_preferences")
        return { language_preference: "auto", effective_language: "eng", spoiler_mode: "free", hints_enabled: false };
      if (command === "get_profile_summary")
        return { active_profile: "ari", profiles: ["ari"] };
      if (command === "get_journal_snapshot") return testJournal;
      if (command === "poll_live_tracking") return 0;
      if (command === "companion_log_available") return companion;
      if (command === "prepare_journal" || command === "start_live_tracking")
        return;
      throw new Error("Unexpected command " + command);
    });
    render(
      <App
        nativeRuntime={() => true}
        resolveGameDirectory={async () => "C:/Test/Fields of Mistria"}
        readinessProbe={async () => ({
          catalog: { game_version: "verified", item_count: 3 },
          catalog_approved: true,
          companion_log_found: companion,
        })}
        importExisting={async () => ({
          profile_id: "ari",
          discovered_items: 2,
          imported: false,
        })}
      />,
    );
    await act(async () => {
      await vi.advanceTimersByTimeAsync(1);
    });
    expect(screen.getAllByText("Ari").length).toBeGreaterThan(0);
    fireEvent.click(screen.getByRole("button", { name: "Settings" }));
    companion = true;
    try {
      await act(async () => {
        await vi.advanceTimersByTimeAsync(30000);
      });
      expect(screen.getByTitle("Live tracking ready")).toBeVisible();
    } finally {
      vi.useRealTimers();
    }
  });

  it("uses the game directory resolved by native code instead of a fixed Steam path", async () => {
    const readinessProbe = vi.fn(async () => ({
      catalog: { game_version: "unapproved", item_count: 0 },
      catalog_approved: false,
      companion_log_found: false,
    }));
    vi.mocked(invoke).mockImplementation(async (command) => {
      if (command === "get_preferences")
        return {
          language_preference: "auto",
          effective_language: "eng",
          spoiler_mode: "free",
          hints_enabled: false,
        };
      throw new Error("Unexpected command " + command);
    });

    render(
      <App
        nativeRuntime={() => true}
        readinessProbe={readinessProbe}
        resolveGameDirectory={async () => "D:/Games/Fields of Mistria"}
      />,
    );

    await waitFor(() =>
      expect(readinessProbe).toHaveBeenCalledWith(
        "D:/Games/Fields of Mistria",
        "C:/Test/mod_data",
      ),
    );
  });

  it("shows one folder action when automatic discovery cannot find the game", async () => {
    vi.mocked(invoke).mockImplementation(async (command) => {
      if (command === "get_preferences")
        return { language_preference: "auto", effective_language: "eng", spoiler_mode: "free", hints_enabled: false };
      throw new Error("Unexpected command " + command);
    });

    render(
      <App
        nativeRuntime={() => true}
        resolveGameDirectory={async () => null}
      />,
    );

    await waitFor(() => expect(screen.getByRole("button", { name: "Settings" })).toBeEnabled());
    fireEvent.click(screen.getByRole("button", { name: "Settings" }));
    expect(
      screen.getByRole("button", { name: "Choose Fields of Mistria folder" }),
    ).toBeVisible();
  });
  it("uses donation status, not acquisition status, for Museum filters", () => {
    preview();
    nav("Museum");
    fireEvent.change(screen.getByLabelText("Donated"), {
      target: { value: "found" },
    });
    expect(
      screen.queryByRole("button", { name: "Trout · Not donated yet" }),
    ).not.toBeInTheDocument();
  });
  it("does not offer seasonal filtering for villagers", () => {
    preview();
    nav("Villagers");
    expect(screen.getByLabelText("Season")).toBeDisabled();
  });
  afterEach(() => {
    cleanup();
    vi.restoreAllMocks();
    vi.mocked(invoke).mockReset();
  });
  it("shows real profile progress and opens its category", () => {
    preview();
    expect(screen.getAllByText("Ari").length).toBeGreaterThan(0);
    nav("Encyclopaedia");
    fireEvent.click(screen.getByRole("button", { name: /Fish 2/ }));
    expect(screen.getByRole("heading", { name: "Fish" })).toBeVisible();
  });
  it("offers multiple partial matches without exposing a hidden item", () => {
    preview();
    search("Trou");
    expect(
      within(screen.getByRole("listbox")).getAllByRole("option"),
    ).toHaveLength(2);
    expect(screen.getByRole("listbox")).not.toHaveTextContent("Undiscovered");
  });
  it("keeps a matching museum selection in Museum", () => {
    preview();
    nav("Museum");
    search("Trou");
    fireEvent.click(screen.getByRole("option", { name: /^Trout/ }));
    expect(screen.getByRole("heading", { name: "Museum" })).toBeVisible();
    expect(screen.getByRole("heading", { name: "Trout" })).toBeVisible();
  });
  it("routes an elsewhere result to its encyclopedia category", () => {
    preview();
    nav("Museum");
    search("rainbow");
    fireEvent.click(within(screen.getByRole("listbox")).getByRole("option"));
    expect(screen.getByRole("heading", { name: "Fish" })).toBeVisible();
    expect(
      screen.getByRole("heading", { name: "Rainbow Trout" }),
    ).toBeVisible();
  });
  it("does not discard an edited note when navigation is cancelled", () => {
    preview();
    search("trout");
    fireEvent.click(screen.getByRole("option", { name: /^Trout/ }));
    fireEvent.change(screen.getByRole("textbox", { name: "Your notes" }), {
      target: { value: "Remember the pond" },
    });
    vi.spyOn(window, "confirm").mockReturnValue(false);
    nav("Villagers");
    expect(screen.getByRole("textbox", { name: "Your notes" })).toHaveValue(
      "Remember the pond",
    );
  });
  it("filters villagers by encountered status", () => {
    preview();
    nav("Villagers");
    fireEvent.change(screen.getByLabelText("Discovered"), {
      target: { value: "missing" },
    });
    expect(
      screen.queryByRole("heading", { name: "Celine" }),
    ).not.toBeInTheDocument();
    expect(
      screen.getByRole("heading", { name: "Someone to meet" }),
    ).toBeVisible();
  });
  it("gives a hidden fish a useful cave clue without naming it", () => {
    preview();
    fireEvent.click(screen.getByRole("button", { name: "Settings" }));
    fireEvent.click(screen.getByRole("switch", { name: /gentle hints/i }));
    nav("Encyclopaedia");
    fireEvent.click(screen.getByRole("button", { name: /Fish 2 \/ 3/ }));
    fireEvent.click(screen.getByRole("button", { name: /^Undiscovered/ }));
    expect(screen.getByRole("dialog")).toHaveTextContent(/cave/i);
    expect(screen.getByRole("dialog")).not.toHaveTextContent("Trout");
  });
  it("switches navigation and search labels to French", () => {
    preview();
    fireEvent.click(screen.getByRole("button", { name: "Settings" }));
    fireEvent.click(screen.getByRole("button", { name: "Français" }));
    expect(screen.getByRole("combobox", { name: /Rechercher/ })).toBeVisible();
    expect(screen.getByRole("button", { name: "Villageois" })).toBeVisible();
  });
  it("does not import a save automatically before the companion log exists", async () => {
    let imported = false;
    vi.mocked(invoke).mockImplementation(async (command) => {
      if (command === "get_preferences")
        return { language_preference: "auto", effective_language: "eng", spoiler_mode: "free", hints_enabled: false };
      if (command === "get_profile_summary")
        return { active_profile: "ari", profiles: ["ari"] };
      if (command === "get_journal_snapshot") return testJournal;
      if (command === "prepare_journal") return;
      throw new Error("Unexpected command " + command);
    });
    render(
      <App
        nativeRuntime={() => true}
        resolveGameDirectory={async () => "C:/Test/Fields of Mistria"}
        readinessProbe={async () => ({
          catalog: { game_version: "verified", item_count: 3 },
          catalog_approved: true,
          companion_log_found: false,
        })}
        importExisting={async () => {
          imported = true;
          return { profile_id: "ari", discovered_items: 2, imported: true };
        }}
      />,
    );
    await waitFor(() =>
      expect(screen.getAllByText("Ari").length).toBeGreaterThan(0),
    );
    expect(
      screen.queryByText("Preparing your journal"),
    ).not.toBeInTheDocument();
    expect(imported).toBe(false);
  });

  it("starts passive companion tracking without opening a game save", async () => {
    vi.mocked(invoke).mockImplementation(async (command) => {
      if (command === "get_preferences")
        return { language_preference: "auto", effective_language: "eng", spoiler_mode: "free", hints_enabled: false };
      if (command === "get_profile_summary")
        return { active_profile: "ari", profiles: ["ari"] };
      if (command === "get_journal_snapshot") return testJournal;
      if (command === "start_live_tracking") return;
      throw new Error("Unexpected command " + command);
    });
    render(
      <App
        nativeRuntime={() => true}
        resolveGameDirectory={async () => "C:/Test/Fields of Mistria"}
        readinessProbe={async () => ({
          catalog: { game_version: "verified", item_count: 3 },
          catalog_approved: true,
          companion_log_found: true,
        })}
      />,
    );

    await waitFor(() => expect(screen.getAllByText("Ari").length).toBeGreaterThan(0));
  });

  it("explains how to refresh an older unsupported save without changing it", async () => {
    vi.mocked(invoke).mockImplementation(async (command) => {
      if (command === "get_preferences")
        return { language_preference: "auto", effective_language: "eng", spoiler_mode: "free", hints_enabled: false };
      if (command === "get_profile_summary")
        return { active_profile: "ari", profiles: ["ari"] };
      if (command === "get_journal_snapshot") return testJournal;
      if (command === "prepare_journal") return;
      throw new Error("Unexpected command " + command);
    });

    render(
      <App
        nativeRuntime={() => true}
        resolveGameDirectory={async () => "C:/Test/Fields of Mistria"}
        readinessProbe={async () => ({
          catalog: { game_version: "verified", item_count: 3 },
          catalog_approved: true,
          companion_log_found: false,
        })}
        importExisting={async () => ({
          status: "unsupported_version",
          profile_id: "ari",
          discovered_items: 0,
          imported: false,
          game_version: "0.14.4",
        })}
      />,
    );

    await waitFor(() => expect(screen.getAllByText("Ari").length).toBeGreaterThan(0));
    fireEvent.click(screen.getByRole("button", { name: "Settings" }));
    fireEvent.click(screen.getByRole("button", { name: "Load save file" }));

    expect(
      await screen.findByText(
        "Save version 0.14.4 is not approved for import. Open this character in the current Fields of Mistria version, save and close the game, then select the updated .sav file. Your original save was not changed.",
      ),
    ).toBeVisible();
  });
});
