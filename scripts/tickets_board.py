#!/usr/bin/env python3
"""Render tickets/*.md into a static Kanban board at tickets/board.html."""

import html
from pathlib import Path

TICKETS_DIR = Path(__file__).resolve().parent.parent / "tickets"
OUTPUT = TICKETS_DIR / "board.html"

COLUMNS = ["backlog", "todo", "in-progress", "done"]
COLUMN_LABELS = {
    "backlog": "Backlog",
    "todo": "To Do",
    "in-progress": "In Progress",
    "done": "Done",
}


def parse_ticket(path: Path) -> dict:
    text = path.read_text()
    _, _, rest = text.partition("---\n")
    frontmatter, _, body = rest.partition("\n---\n")

    fields = {}
    for line in frontmatter.splitlines():
        if ":" not in line:
            continue
        key, _, value = line.partition(":")
        fields[key.strip()] = value.strip()

    fields["body"] = body.strip()
    fields.setdefault("status", "backlog")
    fields.setdefault("milestone", "")
    return fields


def render_card(ticket: dict) -> str:
    title = html.escape(ticket.get("title", "(untitled)"))
    milestone = html.escape(ticket.get("milestone", ""))
    ticket_id = html.escape(ticket.get("id", "?"))
    body_html = "".join(
        f"<p>{html.escape(para)}</p>"
        for para in ticket.get("body", "").split("\n\n")
        if para.strip()
    )
    badge = f'<span class="badge">{milestone}</span>' if milestone else ""
    return f"""
    <article class="card">
      <header><span class="id">#{ticket_id}</span>{badge}</header>
      <h3>{title}</h3>
      <div class="body">{body_html}</div>
    </article>
    """


def build() -> str:
    tickets = [
        parse_ticket(p)
        for p in sorted(TICKETS_DIR.glob("*.md"))
        if p.name != "README.md"
    ]

    columns_html = []
    for status in COLUMNS:
        cards = [render_card(t) for t in tickets if t.get("status") == status]
        columns_html.append(f"""
        <section class="column">
          <h2>{COLUMN_LABELS[status]} <span class="count">{len(cards)}</span></h2>
          <div class="cards">{''.join(cards) if cards else '<p class="empty">Nothing here</p>'}</div>
        </section>
        """)

    return f"""<!doctype html>
<html>
<head>
<meta charset="utf-8">
<title>Koch — Tickets</title>
<style>
  :root {{
    --primary: #8B6F47;
    --primary-dark: #C9A875;
    --ground: #1A1310;
    --fg: #F4EEDD;
  }}
  * {{ box-sizing: border-box; }}
  body {{
    margin: 0;
    padding: 24px;
    background: var(--ground);
    color: var(--fg);
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
  }}
  h1 {{ font-size: 1.25rem; font-weight: 600; margin: 0 0 20px; }}
  .board {{
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: 16px;
  }}
  .column {{
    background: rgba(244, 238, 221, 0.04);
    border: 1px solid rgba(244, 238, 221, 0.1);
    border-radius: 10px;
    padding: 12px;
    min-width: 0;
  }}
  .column h2 {{
    font-size: 0.85rem;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--primary-dark);
    margin: 4px 8px 12px;
    display: flex;
    justify-content: space-between;
  }}
  .count {{ opacity: 0.6; font-weight: 400; }}
  .cards {{ display: flex; flex-direction: column; gap: 10px; }}
  .card {{
    background: #241c17;
    border: 1px solid rgba(244, 238, 221, 0.08);
    border-radius: 8px;
    padding: 12px;
  }}
  .card header {{
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 6px;
  }}
  .card .id {{ font-size: 0.75rem; opacity: 0.5; }}
  .card h3 {{ font-size: 0.95rem; margin: 0 0 6px; font-weight: 600; }}
  .card .body {{ font-size: 0.8rem; line-height: 1.4; opacity: 0.8; }}
  .card .body p {{ margin: 0 0 6px; }}
  .card .body p:last-child {{ margin-bottom: 0; }}
  .badge {{
    font-size: 0.7rem;
    background: var(--primary);
    color: var(--ground);
    padding: 2px 8px;
    border-radius: 999px;
    font-weight: 600;
  }}
  .empty {{ opacity: 0.4; font-size: 0.8rem; font-style: italic; margin: 0 8px; }}
  @media (max-width: 900px) {{
    .board {{ grid-template-columns: 1fr 1fr; }}
  }}
  @media (max-width: 520px) {{
    .board {{ grid-template-columns: 1fr; }}
  }}
</style>
</head>
<body>
  <h1>Koch — Tickets</h1>
  <div class="board">
    {''.join(columns_html)}
  </div>
</body>
</html>
"""


if __name__ == "__main__":
    OUTPUT.write_text(build())
    print(f"wrote {OUTPUT}")
