// @vitest-environment jsdom
import { TestTube } from "lucide-react"
import { BrowserRouter } from "react-router-dom"
import SidebarButton from "../SidebarButton"
import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it } from "vitest";
import '@testing-library/jest-dom/vitest';

afterEach(cleanup);

const TestComponent = () => {
  return (
    <BrowserRouter>
      <SidebarButton to={"/test"} icon={<TestTube />} />
    </BrowserRouter>
  )
}

describe('Sidebar button tests', () => {

  it('Test that Sidebar button renders', () => {

    render(<TestComponent />)
    const component = screen.getByTestId('sidebar-button')
    expect(component).toBeInTheDocument()
  })

  it('Test that Sidebar click works', () => {

    render(<TestComponent />)
    const component = screen.getByTestId('sidebar-button')

    fireEvent.click(component)

    expect(window.location.pathname).toBe("/test");
  })
})