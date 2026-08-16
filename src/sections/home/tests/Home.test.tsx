// @vitest-environment jsdom
import Home from "../Home";
import { STAT_CARDS } from "../mock";
import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import "@testing-library/jest-dom/vitest";

const HOME_CARD_TEST_ID = "home-card";

describe("Home tests", () => {
  afterEach(cleanup);

  it("Test that Home renders the greeting message", () => {
    render(<Home />);

    expect(screen.getByText("Welcome back, player")).toBeInTheDocument();
  });

  it("Test that Home renders a HomeCard for every stat card in mock data", () => {
    render(<Home />);

    const cards = screen.getAllByTestId(HOME_CARD_TEST_ID);

    expect(cards).toHaveLength(STAT_CARDS.length);
  });

  it("Test that Home renders each stat card's label", () => {
    render(<Home />);

    STAT_CARDS.forEach((card) => {
      expect(screen.getByText(card.label)).toBeInTheDocument();
    });
  });
});
