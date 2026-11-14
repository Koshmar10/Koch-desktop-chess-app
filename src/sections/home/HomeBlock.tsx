import type { ReactNode } from "react";
import { HOME_CARD_CLASS } from "./homeStyles";

interface HomeBlockProps {
  title: string;
  subtitle?: string;
  children: ReactNode;
  className?: string;
}

/** One titled card of the Home page, with its small caption beneath. */
const HomeBlock = ({
  title,
  subtitle,
  children,
  className = "",
}: HomeBlockProps) => (
  <section
    className={`flex min-w-0 flex-col gap-3 ${HOME_CARD_CLASS} ${className}`}
  >
    <div>
      <h2 className="text-lg font-semibold text-foreground">{title}</h2>
      {subtitle && <p className="text-xs text-foreground/50">{subtitle}</p>}
    </div>
    {children}
  </section>
);

export default HomeBlock;
