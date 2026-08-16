import { LucideIcon } from "lucide-react";
import { NavLink } from "react-router-dom";

const ICON_SIZE_PX = 24;

interface SidebarButtonProps {
  to: string;
  icon: LucideIcon;
}

const SidebarButton = ({ to, icon: Icon }: SidebarButtonProps) => {
  return (
    <NavLink
      to={to}
      end={to === "/"}
      className={({ isActive }) =>
        `flex justify-center items-center w-fit h-fit py-3 px-3 rounded-lg transition-colors duration-300 ease-in-out ${
          isActive ? "bg-accent/80" : "hover:bg-accent/50"
        }`
      }
      onClick={() => console.log(window.location.pathname)}
      data-testid="sidebar-button"
    >
      <Icon size={ICON_SIZE_PX} />
    </NavLink>
  );
};

export default SidebarButton;
