import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, expect, it } from "vitest";
import { localeOptions } from "../i18n";
import { HintCard } from "./HintCard";
import type { Hint } from "./types";

afterEach(cleanup);

const hint = (changes: Partial<Hint> = {}): Hint => ({
  kind: "fish",
  activity: null,
  areas: [],
  seasons: [],
  source: null,
  ...changes,
});

it("shows the verified area and season without naming the hidden fish", () => {
  render(<HintCard language="eng" hint={hint({ areas: ["pond"], seasons: ["summer"] })} onClose={() => {}} />);
  expect(screen.getByText("Pond")).toBeVisible();
  expect(screen.getByText("Summer")).toBeVisible();
  expect(screen.queryByText("secret_fish")).not.toBeInTheDocument();
});

it("distinguishes a shop recipe from a generic recipe clue", () => {
  render(<HintCard language="eng" hint={hint({ kind: "recipes", source: "store" })} onClose={() => {}} />);
  expect(screen.getByText(/buy.*store/i)).toBeVisible();
});

it("omits unverified area and season instead of guessing", () => {
  render(<HintCard language="eng" hint={hint()} onClose={() => {}} />);
  expect(screen.queryByText("Pond")).not.toBeInTheDocument();
  expect(screen.queryByText("Summer")).not.toBeInTheDocument();
  expect(screen.getByText(/water/i)).toBeVisible();
});

it("offers a gift activity without exposing the recipient's reaction", () => {
  render(<HintCard language="eng" hint={hint({ kind: "gift", activity: "crops", seasons: ["summer"] })} onClose={() => {}} />);
  expect(screen.getByText(/crops/i)).toBeVisible();
  expect(screen.getByText("Summer")).toBeVisible();
  expect(screen.getByRole("heading", { name: "A little nudge" })).toBeVisible();
  expect(screen.queryByText(/loved|liked/i)).not.toBeInTheDocument();
});

it("shows non-crop gift activities when the game validates them", () => {
  render(<HintCard language="eng" hint={hint({ kind: "gift", activity: "dishes" })} onClose={() => {}} />);
  expect(screen.getByText(/dishes/i)).toBeVisible();
});

it("renders each supported language without exposing raw hint codes", () => {
  for (const locale of localeOptions) {
    const { container, unmount } = render(<HintCard language={locale.value} hint={hint({ kind: "recipes", source: "store", areas: ["beach"] })} onClose={() => {}} />);
    expect(container.textContent).not.toMatch(/hint_|\bsecret_fish\b/);
    expect(container.textContent).not.toContain("beach");
    unmount();
  }
});
