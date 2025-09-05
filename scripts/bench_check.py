#!/usr/bin/env python3
import argparse
import json
import os
import sys


def main():
    parser = argparse.ArgumentParser(description="Check criterion benchmark against threshold (ms)")
    parser.add_argument("--name", default="engine_render_default", help="Benchmark function name")
    parser.add_argument("--threshold-ms", type=float, default=50.0, help="Max allowed mean (ms)")
    parser.add_argument("--path", default=None, help="Path to estimates.json (optional)")
    args = parser.parse_args()

    if args.path is None:
        args.path = os.path.join(
            "target",
            "criterion",
            args.name,
            "new",
            "estimates.json",
        )

    if not os.path.exists(args.path):
        print(f"estimates.json not found at {args.path}. Run 'cargo bench' first.")
        return 2

    with open(args.path, "r") as f:
        data = json.load(f)

    # Criterion stores times in nanoseconds by default
    mean_ns = data.get("mean", {}).get("point_estimate")
    if mean_ns is None:
        print("Could not read mean.point_estimate from estimates.json")
        return 3

    mean_ms = mean_ns / 1_000_000.0
    threshold = args.threshold_ms

    print(f"mean: {mean_ms:.3f} ms (threshold {threshold:.3f} ms)")
    if mean_ms > threshold:
        print("FAIL: benchmark exceeds threshold")
        return 1
    print("OK: benchmark within threshold")
    return 0


if __name__ == "__main__":
    sys.exit(main())

