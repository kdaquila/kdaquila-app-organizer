"""Run the room algebra over one or more Python packages."""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

from room_algebra.features.analysis.utils.build_graph import build_graph
from room_algebra.features.analysis.utils.classify import classify
from room_algebra.features.analysis.utils.fixpoint import fixpoint
from room_algebra.features.report.utils.render_report import render_report


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("packages", nargs="+", type=Path)
    parser.add_argument("--depth", type=int, default=1, help="folders below the package root")
    parser.add_argument("--skip-tests", action="store_true")
    parser.add_argument("--show", type=int, default=8)
    args = parser.parse_args()

    for package in args.packages:
        if not package.is_dir():
            print(f"skipping {package}: not a directory", file=sys.stderr)
            continue
        graph = build_graph(package, args.depth, args.skip_tests)
        proposals = classify(graph)
        original = dict(graph.room)
        result = fixpoint(graph)
        graph.room = original
        print(render_report(f"{package.name} (depth {args.depth})", graph, proposals, result, args.show))

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
