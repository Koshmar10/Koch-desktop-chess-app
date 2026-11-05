// @vitest-environment jsdom
import Home from "../Home";
import { cleanup, render, screen } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import { afterEach, describe, expect, it } from "vitest";
import "@testing-library/jest-dom/vitest";

const renderHome = () =>
  render(
    <MemoryRouter>
      <Home />
    </MemoryRouter>,
  );

describe("Home tests", () => {
  afterEach(cleanup);

  it("Test that Home renders each section's title", () => {
    renderHome();

    [
      "Form guide",
      "Your army",
      "At the board",
      "Puzzle record",
      "This week's moments",
      "The scoresheet",
      "Opening lines",
    ].forEach((title) => expect(screen.getByText(title)).toBeInTheDocument());
  });
});
