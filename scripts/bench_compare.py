#!/usr/bin/env python3
"""Compare execution speed of pure-Python slugify vs Rust PyO3 extension.

Usage: run from repository root (script will resolve paths):

. .venv/bin/activate
python scripts/bench_compare.py
"""
from __future__ import annotations

import os
import sys
import time
from statistics import mean

# Resolve paths
SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
REPO_ROOT = os.path.abspath(os.path.join(SCRIPT_DIR, ".."))
# python-slugify is a sibling directory of the repo root
PYTHON_SLUGIFY_ROOT = os.path.abspath(os.path.join(REPO_ROOT, "..", "python-slugify"))
if os.path.isdir(PYTHON_SLUGIFY_ROOT):
    sys.path.insert(0, PYTHON_SLUGIFY_ROOT)
else:
    print(f"Warning: expected python-slugify at {PYTHON_SLUGIFY_ROOT} not found.\nMake sure repository layout matches.")

# Import pure Python implementation
try:
    from slugify.slugify import slugify as py_slugify
    from slugify.slugify import DEFAULT_SEPARATOR as PY_DEFAULT_SEPARATOR
except Exception as e:
    print("Failed to import pure-Python slugify:", e)
    raise

# Import Rust PyO3 extension
try:
    import python_slugify_pi
    rs_slugify = python_slugify_pi.slugify
except Exception as e:
    print("Failed to import PyO3 extension python_slugify_pi:", e)
    raise

# Prepare inputs (sanitized: avoid emojis/symbols with differing transliteration)
N = 2500
inputs = [f"This is a test - {i} Äpfel & Öl -- 123 &amp; {i}" for i in range(N)]

# Warmup to ensure any lazy initialisation done (use explicit identical args)
call_kwargs = dict(
    entities=True,
    decimal=True,
    hexadecimal=True,
    max_length=0,
    word_boundary=False,
    separator=PY_DEFAULT_SEPARATOR,
    save_order=False,
    stopwords=None,
    regex_pattern=None,
    lowercase=True,
    replacements=None,
    allow_unicode=False,
)
for s in inputs[:50]:
    _ = py_slugify(s)
    try:
        _ = rs_slugify(s, **call_kwargs)
    except TypeError:
        # fallback if rust binding doesn't accept kwargs in this environment
        _ = rs_slugify(s)

# Verify outputs match for a small sample
mismatches = []
for s in inputs[:100]:
    a = py_slugify(s)
    try:
        b = rs_slugify(s, **call_kwargs)
    except TypeError:
        b = rs_slugify(s)
    if a != b:
        mismatches.append((s, a, b))

print(f"Checked first 100 samples: {len(mismatches)} mismatches")
if mismatches:
    print("Showing up to 10 mismatches:")
    for s, a, b in mismatches[:10]:
        print('INPUT:', s)
        print('PY   :', a)
        print('RUST :', b)
        print('-' * 40)

# Benchmark function that returns times per call list

def bench(func, data, repeat=1):
    """Run the provided function over `data` repeat times and measure totals.

    Returns list of total durations for each repeat.
    """
    totals = []
    for _ in range(repeat):
        t0 = time.perf_counter()
        for s in data:
            func(s)
        t1 = time.perf_counter()
        totals.append(t1 - t0)
    return totals

REPEAT = 3
print(f"Running benchmark: {N} calls, {REPEAT} repeats")
py_totals = bench(lambda s: py_slugify(s), inputs, repeat=REPEAT)
rs_totals = bench(lambda s: rs_slugify(s, **call_kwargs) if hasattr(rs_slugify, '__call__') else rs_slugify(s), inputs, repeat=REPEAT)

print('\nPure Python totals (s):', ['{:.6f}'.format(t) for t in py_totals])
print('Rust extension totals (s):', ['{:.6f}'.format(t) for t in rs_totals])

py_avg = mean(py_totals)
rs_avg = mean(rs_totals)
per_call_py = py_avg / N
per_call_rs = rs_avg / N

print(f"\nAverage total: Python {py_avg:.6f}s, Rust {rs_avg:.6f}s")
print(
    f"Per call average: Python {per_call_py*1e6:.2f} μs,"
    f" Rust {per_call_rs*1e6:.2f} μs",
)
if rs_avg > 0:
    print(f"Speedup (Python / Rust): {py_avg/rs_avg:.2f}x")

# Quick sanity: print sample output
print('\nSample output:')
print('py :', py_slugify(inputs[0]))
print('rs :', rs_slugify(inputs[0]))

print('\nDone')
