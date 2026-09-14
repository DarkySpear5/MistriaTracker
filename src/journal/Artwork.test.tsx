import { cleanup, render, waitFor } from "@testing-library/react";
import { afterEach, expect, it } from "vitest";
import { Artwork, ArtworkProvider } from "./Artwork";
afterEach(cleanup);
it("reloads a revealed icon rather than retaining its cached black silhouette", async () => {
  let revealed = false;
  const loader = async () => ({
    e1: revealed
      ? "data:image/png;base64,COLOR"
      : "data:image/png;base64,BLACK",
  });
  const tree = (hidden: boolean) => (
    <ArtworkProvider loader={loader} scope="ari">
      <Artwork token="e1" hidden={hidden} />
    </ArtworkProvider>
  );
  const { container, rerender } = render(tree(true));
  await waitFor(() =>
    expect(container.querySelector("img")).toHaveAttribute(
      "src",
      "data:image/png;base64,BLACK",
    ),
  );
  revealed = true;
  rerender(tree(false));
  await waitFor(() =>
    expect(container.querySelector("img")).toHaveAttribute(
      "src",
      "data:image/png;base64,COLOR",
    ),
  );
});
