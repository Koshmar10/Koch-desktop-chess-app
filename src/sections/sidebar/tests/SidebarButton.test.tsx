// @vitest-environment jsdom
import { TestTube } from "lucide-react";
import { BrowserRouter } from "react-router-dom";
import SidebarButton from "../SidebarButton";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import "@testing-library/jest-dom/vitest";

const TEST_PATH = "/test";
const SIDEBAR_BUTTON_TEST_ID = "sidebar-button";

const TestComponent = () => {
  return (
    <BrowserRouter>
      <SidebarButton to={TEST_PATH} icon={TestTube} />
    </BrowserRouter>
  );
};

describe("Sidebar button tests", () => {
  afterEach(cleanup);

  it("Test that Sidebar button renders", () => {
    render(<TestComponent />);

    const component = screen.getByTestId(SIDEBAR_BUTTON_TEST_ID);

    expect(component).toBeInTheDocument();
  });

  it("Test that Sidebar click works", () => {
    render(<TestComponent />);
    const component = screen.getByTestId(SIDEBAR_BUTTON_TEST_ID);

    fireEvent.click(component);

    expect(window.location.pathname).toBe(TEST_PATH);
  });
});
