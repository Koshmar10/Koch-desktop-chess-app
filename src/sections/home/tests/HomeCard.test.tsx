import { TrendingUp } from "lucide-react";
import HomeCard from "../HomeCard";
import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import "@testing-library/jest-dom/vitest";

const HOME_CARD_TEST_ID = "home-card";

describe("HomeCard tests", () => {
  afterEach(cleanup);

  it("Test that HomeCard renders", () => {
    render(<HomeCard icon={TrendingUp} value={1450} label="Current Rating" />);

    const component = screen.getByTestId(HOME_CARD_TEST_ID);

    expect(component).toBeInTheDocument();
  });

  it("Test that HomeCard renders value and label", () => {
    render(<HomeCard icon={TrendingUp} value={1450} label="Current Rating" />);

    expect(screen.getByText("1450")).toBeInTheDocument();
    expect(screen.getByText("Current Rating")).toBeInTheDocument();
  });

  it("Test that HomeCard renders delta when provided", () => {
    render(
      <HomeCard
        icon={TrendingUp}
        value={1450}
        label="Current Rating"
        delta="+32 this month"
      />,
    );

    expect(screen.getByText("+32 this month")).toBeInTheDocument();
  });

  it("Test that HomeCard omits delta when not provided", () => {
    render(<HomeCard icon={TrendingUp} value={1450} label="Current Rating" />);

    expect(screen.queryByText(/this month/)).not.toBeInTheDocument();
  });

  it("Test that HomeCard applies custom className", () => {
    render(
      <HomeCard
        icon={TrendingUp}
        value={1450}
        label="Current Rating"
        className="row-span-2 col-span-1"
      />,
    );

    const component = screen.getByTestId(HOME_CARD_TEST_ID);

    expect(component).toHaveClass("row-span-2", "col-span-1");
  });

  it("Test that HomeCard renders the icon", () => {
    const { container } = render(
      <HomeCard icon={TrendingUp} value={1450} label="Current Rating" />,
    );

    expect(container.querySelector("svg")).toBeInTheDocument();
  });
});
