#!/usr/bin/env python3
"""Render the complete Flow-suite issue atlas from its checked-in snapshot.

This is an offline document generator. It does not query GitHub, infer issue
completion, execute checks, or modify the snapshot. Every manifest node gets
one card; dependency edges belong to the companion Markdown/Mermaid map.
"""

from __future__ import annotations

import argparse
from collections import Counter, OrderedDict
from dataclasses import dataclass
from functools import lru_cache
import json
import math
from pathlib import Path
import re
from typing import Any
import xml.etree.ElementTree as ET

import matplotlib

matplotlib.use("Agg")
from matplotlib import pyplot as plt
from matplotlib.font_manager import FontProperties
from matplotlib.patches import FancyBboxPatch, Rectangle
from matplotlib.textpath import TextPath


REPOSITORIES = ("flow", "renderflow", "optiflow", "aniflow")
REPOSITORY_NAMES = {
    "flow": "Flow",
    "renderflow": "Renderflow",
    "optiflow": "Optiflow",
    "aniflow": "Aniflow",
}
STATUS = {
    "nuked": ("NUKED", "#147D53", "#E7F5EB"),
    "draft": ("DRAFT", "#087F9B", "#E2F4FA"),
    "qualification": ("CHECKS / RELEASE OWED", "#9C6100", "#FFF1CE"),
    "blocked": ("BLOCKED", "#B44044", "#FBE9E8"),
    "planned": ("PLANNED", "#4C6688", "#ECF0F7"),
    "epic": ("EPIC", "#7454A0", "#F0EBF7"),
    "external": ("EXTERNAL GATE", "#666D79", "#ECEFF2"),
    "optional": ("OPTIONAL", "#3D7775", "#EAF4F1"),
    "horizon": ("FUTURE HORIZON", "#85647C", "#F4ECF2"),
}

BACKGROUND = "#F3F5F8"
INK = "#162A40"
MUTED = "#54657A"
HEADER = "#14283F"
BORDER = "#DCE3EB"
FIGURE_WIDTH = 24.4
OUTER_MARGIN = 0.42
COLUMN_GAP = 0.24
COLUMN_WIDTH = (FIGURE_WIDTH - 2 * OUTER_MARGIN - 3 * COLUMN_GAP) / 4
HEADER_HEIGHT = 3.08
REPOSITORY_HEADER_HEIGHT = 1.52
GROUP_HEADER_HEIGHT = 0.40
CARD_GAP = 0.12
GROUP_GAP = 0.16
FOOTER_HEIGHT = 0.66
TITLE_SIZE = 13.2
TITLE_LEADING = 0.225
NOTE_SIZE = 10.4
NOTE_LEADING = 0.179
SVG_NS = "http://www.w3.org/2000/svg"


@dataclass(frozen=True)
class Card:
    node: dict[str, Any]
    title: tuple[str, ...]
    note: tuple[str, ...]
    height: float
    gid: str


@dataclass(frozen=True)
class Group:
    name: str
    cards: tuple[Card, ...]


@lru_cache(maxsize=8192)
def text_width(value: str, size: float, bold: bool = False) -> float:
    """Return text width in inches using the same bundled font as rendering."""
    if not value:
        return 0.0
    properties = FontProperties(
        family="DejaVu Sans", size=size, weight="bold" if bold else "normal"
    )
    width = TextPath((0, 0), value, prop=properties).get_extents().width
    return max(0.0, width / 72) if math.isfinite(width) else 0.0


def wrap(
    value: Any, width: float, size: float, *, bold: bool = False, limit: int | None = None
) -> tuple[str, ...]:
    words = str(value or "").split()
    lines: list[str] = []
    current = ""
    for word in words:
        candidate = f"{current} {word}" if current else word
        if text_width(candidate, size, bold) <= width:
            current = candidate
            continue
        if current:
            lines.append(current)
            current = ""
        # Long unbroken identifiers are still contained within the card.
        while text_width(word, size, bold) > width:
            cut = len(word) - 1
            while cut > 1 and text_width(word[:cut], size, bold) > width:
                cut -= 1
            lines.append(word[:cut])
            word = word[cut:]
        current = word
    if current:
        lines.append(current)
    if limit is not None and len(lines) > limit:
        lines = lines[:limit]
        last = lines[-1]
        while last and text_width(last + "…", size, bold) > width:
            last = last[:-1].rstrip()
        lines[-1] = last + "…"
    return tuple(lines)


def count(value: Any) -> int:
    if isinstance(value, int) and not isinstance(value, bool):
        return value
    if isinstance(value, (list, dict)):
        return len(value)
    raise ValueError("Repository open_issues/open_prs must be counts or collections")


def load_snapshot(path: Path) -> dict[str, Any]:
    snapshot = json.loads(path.read_text(encoding="utf-8"))
    nodes = snapshot["nodes"]
    repositories = snapshot["repositories"]
    if not isinstance(nodes, list) or not isinstance(repositories, dict):
        raise ValueError("Expected nodes list and repositories object")
    seen: set[str] = set()
    for node in nodes:
        identifier = str(node["id"])
        if identifier in seen:
            raise ValueError(f"Duplicate node id: {identifier}")
        seen.add(identifier)
        if node["status"] not in STATUS:
            raise ValueError(f"Unknown status for {identifier}: {node['status']}")
        if not str(node.get("title", "")).strip():
            raise ValueError(f"Node {identifier} has no title")
    for repository in REPOSITORIES:
        record = repositories[repository]
        count(record["open_issues"])
        count(record["open_prs"])
    return snapshot


def layouts(snapshot: dict[str, Any]) -> dict[str, tuple[Group, ...]]:
    columns: dict[str, OrderedDict[str, list[Card]]] = {
        name: OrderedDict() for name in REPOSITORIES
    }
    for index, node in enumerate(snapshot["nodes"]):
        # Preserve an unexpected external-repository node visibly rather than
        # filtering it out. The standard manifest uses only the four columns.
        repository = node.get("repo", "flow")
        column = repository if repository in columns else "flow"
        group = str(node.get("group") or "Other tracked work")
        title = wrap(node["title"], COLUMN_WIDTH - 0.42, TITLE_SIZE, bold=True)
        note = wrap(node.get("note", ""), COLUMN_WIDTH - 0.42, NOTE_SIZE, limit=3)
        height = 0.50 + len(title) * TITLE_LEADING
        if note:
            height += 0.075 + len(note) * NOTE_LEADING
        slug = re.sub(r"[^A-Za-z0-9_.-]", "-", str(node["id"]))
        card = Card(node, title, note, height, f"node-{index:04d}-{slug}")
        columns[column].setdefault(group, []).append(card)
    return {
        repository: tuple(Group(name, tuple(cards)) for name, cards in groups.items())
        for repository, groups in columns.items()
    }


def column_height(groups: tuple[Group, ...]) -> float:
    return REPOSITORY_HEADER_HEIGHT + sum(
        GROUP_HEADER_HEIGHT
        + sum(card.height + CARD_GAP for card in group.cards)
        + GROUP_GAP
        for group in groups
    )


def label(ax: Any, x: float, y: float, value: str, *, size: float = 10,
          color: str = INK, bold: bool = False, url: str | None = None,
          align: str = "left") -> Any:
    artist = ax.text(
        x, y, value, fontsize=size, color=color,
        fontweight="bold" if bold else "normal", ha=align, va="top",
        fontfamily="DejaVu Sans", clip_on=False,
    )
    if url:
        artist.set_url(url)
    return artist


def chip(ax: Any, x: float, y: float, status: str, *, size: float = 8.4,
         url: str | None = None) -> float:
    value, foreground, background = STATUS[status]
    width = text_width(value, size, True) + 0.22
    box = FancyBboxPatch(
        (x, y), width, 0.265, boxstyle="round,pad=0,rounding_size=0.06",
        facecolor=background, edgecolor="none", linewidth=0,
    )
    if url:
        box.set_url(url)
    ax.add_patch(box)
    label(ax, x + 0.11, y + 0.059, value, size=size,
          color=foreground, bold=True, url=url)
    return width


def draw_header(ax: Any, snapshot: dict[str, Any]) -> None:
    nodes = snapshot["nodes"]
    totals = Counter(node["status"] for node in nodes)
    ax.add_patch(Rectangle((0, 0), FIGURE_WIDTH, HEADER_HEIGHT - 0.13,
                           facecolor=HEADER, edgecolor="none"))
    label(ax, OUTER_MARGIN, 0.23, "FLOW SUITE  /  COMPLETE ISSUE ATLAS",
          size=26, color="#FFFFFF", bold=True)
    label(ax, FIGURE_WIDTH - OUTER_MARGIN, 0.32,
          f"SNAPSHOT  {snapshot['snapshot_at']}", size=10.2,
          color="#C2D4E7", align="right")
    label(ax, OUTER_MARGIN, 0.79,
          "Every tracked node stays visible. This atlas groups work; it does not invent execution order.",
          size=12, color="#D6E1ED")
    statistics = (
        (len(nodes), "TRACKED NODES"),
        (totals["nuked"], "NUKED"),
        (sum(count(item["open_issues"]) for item in snapshot["repositories"].values()), "LIVE OPEN ISSUES"),
        (sum(count(item["open_prs"]) for item in snapshot["repositories"].values()), "LIVE OPEN PRs"),
    )
    for index, (value, caption) in enumerate(statistics):
        x = OUTER_MARGIN + index * 4.08
        label(ax, x, 1.20, str(value), size=29, color="#FFFFFF", bold=True)
        label(ax, x + 1.13, 1.34, caption, size=10.2, color="#B6CBDF", bold=True)
    label(ax, OUTER_MARGIN, 1.95,
          "NUKED = bounded issue complete. It does not mean every platform, check, release, or product horizon is complete.",
          size=11, color="#DCE8F3")
    x = OUTER_MARGIN
    for status in STATUS:
        x += chip(ax, x, 2.40, status, size=8.3) + 0.17
    original = sum(bool(node.get("original")) for node in nodes)
    label(ax, FIGURE_WIDTH - OUTER_MARGIN, 2.79,
          f"{original} original nodes retained  ·  Dependency details: companion map",
          size=8.5, color="#BDD0E3", align="right",
          url="flow-suite-dependency-map.md")


def draw_repository_header(ax: Any, x: float, y: float, repository: str,
                           groups: tuple[Group, ...], record: dict[str, Any]) -> None:
    cards = [card for group in groups for card in group.cards]
    completed = sum(card.node["status"] == "nuked" for card in cards)
    label(ax, x, y + 0.04, REPOSITORY_NAMES[repository], size=21, bold=True)
    label(ax, x, y + 0.43,
          f"{len(cards)} tracked  ·  {completed} NUKED  ·  "
          f"{count(record['open_issues'])} open issues  ·  {count(record['open_prs'])} open PRs",
          size=9.2, color=MUTED)
    main = record.get("main", "unrecorded")
    if isinstance(main, dict):
        main = main.get("sha", "unrecorded")
    label(ax, x, y + 0.67, f"MAIN  {str(main)[:12]}", size=8.8, color=MUTED)
    release_lines = wrap(record.get("release_summary", "Release evidence not recorded"),
                         COLUMN_WIDTH - 0.08, 9.2, limit=3)
    for index, line in enumerate(release_lines):
        label(ax, x, y + 0.88 + index * 0.16, line, size=9.2, color=MUTED)
    ax.plot([x, x + COLUMN_WIDTH], [y + 1.41, y + 1.41], color=BORDER, linewidth=1.1)


def draw_card(ax: Any, x: float, y: float, card: Card) -> None:
    node = card.node
    status = node["status"]
    _, color, _ = STATUS[status]
    url = node.get("url") or None
    box = FancyBboxPatch(
        (x, y), COLUMN_WIDTH, card.height,
        boxstyle="round,pad=0,rounding_size=0.09",
        facecolor="#FFFFFF", edgecolor=BORDER, linewidth=0.7,
    )
    box.set_gid(card.gid)
    if url:
        box.set_url(url)
    ax.add_patch(box)
    ax.add_patch(Rectangle((x + 0.035, y + 0.13), 0.045, card.height - 0.26,
                           facecolor=color, edgecolor="none"))
    repository = str(node.get("repo", "flow"))
    number = node.get("number")
    reference = f"{repository.upper()} #{number}" if number is not None else f"{repository.upper()} · {node['id']}"
    label(ax, x + 0.18, y + 0.15, reference, size=9.7,
          color=MUTED, bold=True, url=url)
    status_width = text_width(STATUS[status][0], 8.0, True) + 0.22
    chip(ax, x + COLUMN_WIDTH - status_width - 0.13, y + 0.105,
         status, size=8.0, url=url)
    cursor = y + 0.43
    for line in card.title:
        label(ax, x + 0.18, cursor, line, size=TITLE_SIZE, bold=True, url=url)
        cursor += TITLE_LEADING
    if card.note:
        cursor += 0.075
        for line in card.note:
            label(ax, x + 0.18, cursor, line, size=NOTE_SIZE, color=MUTED, url=url)
            cursor += NOTE_LEADING


def enrich_svg(path: Path, snapshot: dict[str, Any], columns: dict[str, tuple[Group, ...]]) -> None:
    """Keep real SVG text and add full, untruncated per-node accessible titles."""
    ET.register_namespace("", SVG_NS)
    ET.register_namespace("xlink", "http://www.w3.org/1999/xlink")
    ET.register_namespace("dc", "http://purl.org/dc/elements/1.1/")
    ET.register_namespace("cc", "http://creativecommons.org/ns#")
    ET.register_namespace("rdf", "http://www.w3.org/1999/02/22-rdf-syntax-ns#")
    tree = ET.parse(path)
    root = tree.getroot()
    root.set("role", "img")
    root.set("aria-labelledby", "atlas-title atlas-description")
    title = ET.Element(f"{{{SVG_NS}}}title", {"id": "atlas-title"})
    title.text = f"Flow suite complete issue atlas — {snapshot['snapshot_at']}"
    description = ET.Element(f"{{{SVG_NS}}}desc", {"id": "atlas-description"})
    description.text = (
        f"All {len(snapshot['nodes'])} manifest nodes, grouped by repository and work group. "
        "NUKED means the bounded issue is complete, not that all checks or releases are complete. "
        "Cards link to their source issues. Dependency relationships are in the companion map."
    )
    root.insert(0, title)
    root.insert(1, description)
    by_gid = {card.gid: card.node for groups in columns.values() for group in groups for card in group.cards}
    for element in root.iter():
        node = by_gid.get(element.get("id", ""))
        if node is None:
            continue
        detail = ET.Element(f"{{{SVG_NS}}}title")
        detail.text = f"{node['id']}: {node['title']} — {STATUS[node['status']][0]}. {node.get('note', '')}"
        element.insert(0, detail)
        element.set("data-node-id", str(node["id"]))
        element.set("data-status", node["status"])
        element.set("data-original", str(bool(node.get("original"))).lower())
    tree.write(path, encoding="utf-8", xml_declaration=True)


def render(snapshot: dict[str, Any], output: Path, max_width: int, max_height: int) -> tuple[Path, Path]:
    columns = layouts(snapshot)
    height = HEADER_HEIGHT + max(column_height(groups) for groups in columns.values()) + FOOTER_HEIGHT
    matplotlib.rcParams.update({
        "font.family": "DejaVu Sans",
        "svg.fonttype": "none",
        "svg.hashsalt": "flow-suite-completion-atlas-v1",
        "axes.unicode_minus": False,
    })
    figure = plt.figure(figsize=(FIGURE_WIDTH, height), facecolor=BACKGROUND)
    ax = figure.add_axes((0, 0, 1, 1))
    ax.set_xlim(0, FIGURE_WIDTH)
    ax.set_ylim(height, 0)
    ax.set_axis_off()
    draw_header(ax, snapshot)
    for index, repository in enumerate(REPOSITORIES):
        x = OUTER_MARGIN + index * (COLUMN_WIDTH + COLUMN_GAP)
        y = HEADER_HEIGHT
        groups = columns[repository]
        draw_repository_header(ax, x, y, repository, groups, snapshot["repositories"][repository])
        y += REPOSITORY_HEADER_HEIGHT
        for group in groups:
            label(ax, x + 0.015, y + 0.045, group.name.upper(), size=10.1, color=MUTED, bold=True)
            label(ax, x + COLUMN_WIDTH - 0.03, y + 0.05, str(len(group.cards)),
                  size=9.3, color=MUTED, align="right")
            y += GROUP_HEADER_HEIGHT
            for card in group.cards:
                draw_card(ax, x, y, card)
                y += card.height + CARD_GAP
            y += GROUP_GAP
    footer_y = height - FOOTER_HEIGHT + 0.12
    label(ax, OUTER_MARGIN, footer_y,
          "SOURCE  flow-suite-status.json   ·   DEPENDENCIES  flow-suite-dependency-map.md   ·   No percentage or release readiness inferred",
          size=9.0, color=MUTED, url="flow-suite-dependency-map.md")
    label(ax, OUTER_MARGIN, footer_y + 0.23,
          "Full notes and source links remain in the manifest; SVG card tooltips retain untruncated notes. All manifest nodes are shown.",
          size=8.5, color=MUTED)
    output.mkdir(parents=True, exist_ok=True)
    svg_path = output / "flow-suite-completion.svg"
    png_path = output / "flow-suite-completion.png"
    # Both outputs use this same figure/layout. DPI alone bounds raster size.
    dpi = min(110.0, (max_width - 2) / FIGURE_WIDTH, (max_height - 2) / height)
    try:
        figure.savefig(svg_path, format="svg", facecolor=BACKGROUND,
                       metadata={"Date": None, "Creator": "Flow suite roadmap atlas"})
        figure.savefig(png_path, format="png", dpi=dpi, facecolor=BACKGROUND,
                       metadata={"Software": "Flow suite roadmap atlas"})
    finally:
        plt.close(figure)
    enrich_svg(svg_path, snapshot, columns)
    return svg_path, png_path


def main() -> None:
    repository_root = Path(__file__).resolve().parents[1]
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--source", type=Path, default=repository_root / "docs/roadmaps/flow-suite-status.json")
    parser.add_argument("--output-dir", type=Path, default=repository_root / "docs/roadmaps")
    parser.add_argument("--max-png-width", type=int, default=2200)
    parser.add_argument("--max-png-height", type=int, default=6000)
    arguments = parser.parse_args()
    if arguments.max_png_width < 400 or arguments.max_png_height < 400:
        parser.error("PNG bounds must each be at least 400 pixels")
    snapshot = load_snapshot(arguments.source)
    paths = render(snapshot, arguments.output_dir, arguments.max_png_width, arguments.max_png_height)
    print(json.dumps({"nodes": len(snapshot["nodes"]), "outputs": [str(path) for path in paths]}, sort_keys=True))


if __name__ == "__main__":
    main()
