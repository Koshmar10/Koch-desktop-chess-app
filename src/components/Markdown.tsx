import { memo, type ReactNode } from "react";
import ReactMarkdown, {
  defaultUrlTransform,
  type Components,
} from "react-markdown";
import remarkGfm from "remark-gfm";
import { openUrl } from "@tauri-apps/plugin-opener";
import { SEGMENT_LINK_PREFIX, segmentIndexOf } from "../lib/segments";

// GitHub-flavoured markdown: tables, strikethrough and task lists on top of
// the basics, all of which models use freely.
const REMARK_PLUGINS = [remarkGfm];

interface ExternalLinkProps {
  href?: string;
  children?: ReactNode;
}

// A plain link would navigate the app's own window away from the app, so
// links open in the system browser instead.
const ExternalLink = ({ href, children }: ExternalLinkProps) => (
  <a
    href={href}
    className="text-primary underline"
    onClick={(e) => {
      e.preventDefault();
      if (href) openUrl(href).catch(console.error);
    }}
  >
    {children}
  </a>
);

// Tailwind's reset strips every element's default look — lists lose their
// bullets, headings their size — so each element gets its classes here.
const MARKDOWN_COMPONENTS: Components = {
  p: ({ children }) => <p className="leading-relaxed">{children}</p>,
  h1: ({ children }) => <h1 className="text-lg font-semibold">{children}</h1>,
  h2: ({ children }) => <h2 className="text-base font-semibold">{children}</h2>,
  h3: ({ children }) => <h3 className="text-base font-semibold">{children}</h3>,
  ul: ({ children }) => (
    <ul className="list-disc space-y-1 pl-5">{children}</ul>
  ),
  ol: ({ children }) => (
    <ol className="list-decimal space-y-1 pl-5">{children}</ol>
  ),
  strong: ({ children }) => (
    <strong className="font-semibold">{children}</strong>
  ),
  em: ({ children }) => <em className="italic">{children}</em>,
  blockquote: ({ children }) => (
    <blockquote className="border-l-2 border-primary/50 pl-3 text-muted-foreground">
      {children}
    </blockquote>
  ),
  code: ({ children }) => (
    <code className="rounded bg-secondary/60 px-1 font-mono text-sm">
      {children}
    </code>
  ),
  // A code block's `code` sits inside this, so its own inline look is undone.
  pre: ({ children }) => (
    <pre className="overflow-x-auto rounded bg-secondary/60 p-2 [&>code]:bg-transparent [&>code]:p-0">
      {children}
    </pre>
  ),
  table: ({ children }) => (
    <div className="overflow-x-auto">
      <table className="text-sm">{children}</table>
    </div>
  ),
  th: ({ children }) => (
    <th className="border border-border/60 px-2 py-1 text-left font-semibold">
      {children}
    </th>
  ),
  td: ({ children }) => (
    <td className="border border-border/60 px-2 py-1">{children}</td>
  ),
  hr: () => <hr className="border-border/60" />,
  a: ExternalLink,
};

// Segment links are an internal scheme, which react-markdown would blank as
// an unsafe URL; every other link still goes through its default check.
const urlTransform = (url: string) =>
  url.startsWith(SEGMENT_LINK_PREFIX) ? url : defaultUrlTransform(url);

interface MarkdownProps {
  // Markdown source, as the model writes it.
  text: string;
  // What each segment link in `text` is shown as, by its position.
  chips?: ReactNode[];
}

/**
 * The assistant's answers, rendered. Raw HTML in the text is left out
 * (react-markdown's default), so an answer can't put markup of its own
 * into the app.
 */
const Markdown = ({ text, chips = [] }: MarkdownProps) => {
  const components: Components = {
    ...MARKDOWN_COMPONENTS,
    a: (props) => {
      const index = segmentIndexOf(props.href);
      return index === null ? (
        <ExternalLink {...props} />
      ) : (
        (chips[index] ?? null)
      );
    },
  };
  return (
    <div className="flex flex-col gap-2 break-words">
      <ReactMarkdown
        remarkPlugins={REMARK_PLUGINS}
        components={components}
        urlTransform={urlTransform}
      >
        {text}
      </ReactMarkdown>
    </div>
  );
};

// Parsing is the expensive part, so an unchanged text and chips skip it.
export default memo(Markdown);
