// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import "@testing-library/jest-dom/vitest";
import Dropdown from "../Dropdown";

const TRIGGER_TEST_ID = "dropdown-trigger";
const PANEL_TEST_ID = "dropdown-panel";
const SELECT_TEST_ID = "dropdown-select";

const TestDropdown = () => (
  <Dropdown
    trigger={({ toggle }) => (
      <button data-testid={TRIGGER_TEST_ID} onClick={toggle}>
        Open
      </button>
    )}
  >
    {(close) => (
      <div data-testid={PANEL_TEST_ID}>
        <button data-testid={SELECT_TEST_ID} onClick={close}>
          Pick me
        </button>
      </div>
    )}
  </Dropdown>
);

describe("Dropdown", () => {
  afterEach(cleanup);

  it("does not render the panel until opened", () => {
    render(<TestDropdown />);

    expect(screen.queryByTestId(PANEL_TEST_ID)).not.toBeInTheDocument();
  });

  it("opens the panel when the trigger is clicked", () => {
    render(<TestDropdown />);

    fireEvent.click(screen.getByTestId(TRIGGER_TEST_ID));

    expect(screen.getByTestId(PANEL_TEST_ID)).toBeInTheDocument();
  });

  it("closes the panel when the trigger is clicked again", () => {
    render(<TestDropdown />);

    fireEvent.click(screen.getByTestId(TRIGGER_TEST_ID));
    fireEvent.click(screen.getByTestId(TRIGGER_TEST_ID));

    expect(screen.queryByTestId(PANEL_TEST_ID)).not.toBeInTheDocument();
  });

  it("closes the panel when the panel content calls close()", () => {
    render(<TestDropdown />);

    fireEvent.click(screen.getByTestId(TRIGGER_TEST_ID));
    fireEvent.click(screen.getByTestId(SELECT_TEST_ID));

    expect(screen.queryByTestId(PANEL_TEST_ID)).not.toBeInTheDocument();
  });

  it("closes the panel on an outside click", () => {
    render(
      <div>
        <TestDropdown />
        <button data-testid="outside">Outside</button>
      </div>,
    );

    fireEvent.click(screen.getByTestId(TRIGGER_TEST_ID));
    fireEvent.mouseDown(screen.getByTestId("outside"));

    expect(screen.queryByTestId(PANEL_TEST_ID)).not.toBeInTheDocument();
  });
});
