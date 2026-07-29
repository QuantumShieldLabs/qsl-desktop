#!/usr/bin/env bash
# NA-0686 / D-1325 (ENG-0075) — the desktop suite must ENUMERATE WHAT IT RAN.
#
# THE DEFECT THIS CLOSES: CI ran `cargo test -q`. The `-q` hides which test
# BINARIES ran and how many tests each contained, so the only visible signal was
# "ok". Delete a test file and the suite still says ok — at a lower total that
# nobody compares against anything. An aggregate summarises; it does not check.
#
# ⚠ THE POINT IS THE COMPARISON, NOT THE PRINTING. Printing counts into a log
# that nobody diffs is the same defect wearing a longer output. So the inventory
# is pinned in `scripts/ci/EXPECTED_TEST_INVENTORY.txt` and compared here, and a
# target that disappears or shrinks FAILS THE BUILD.
#
# Growth is allowed and does not fail: adding tests is the normal case, and a
# gate that fires on new tests would be turned off within a week (the same
# reasoning that put Tier 2a of the literal scan out of scope). Shrinkage and
# disappearance are what the pin exists to catch.
#
# Regenerate deliberately, never reflexively, after a REVIEWED removal:
#     scripts/ci/test_inventory.sh --write
set -uo pipefail

cd "$(dirname "$0")/../.." || exit 2
EXPECTED="scripts/ci/EXPECTED_TEST_INVENTORY.txt"
ACTUAL="$(mktemp)"
trap 'rm -f "$ACTUAL"' EXIT

# `--list` names every test without running it; two trailing lines summarise, so
# they are dropped. `--format terse` keeps the shape stable across cargo versions.
if ! cargo test --no-run --quiet 2>/dev/null; then
  echo "test_inventory: FAILED to build test targets" >&2
  exit 2
fi

cargo test -- --list --format terse 2>/dev/null \
  | grep ': test$' \
  | sed 's/: test$//' \
  | sort > "$ACTUAL.names"

# One line per test NAME, so a removal is visible by name rather than as a
# number that moved. A count alone cannot say WHICH test left.
sort -o "$ACTUAL.names" "$ACTUAL.names"
cp "$ACTUAL.names" "$ACTUAL"

COUNT=$(wc -l < "$ACTUAL" | tr -d ' ')

if [ "${1:-}" = "--write" ]; then
  cp "$ACTUAL" "$EXPECTED"
  echo "test_inventory: pinned $COUNT tests into $EXPECTED"
  exit 0
fi

if [ ! -f "$EXPECTED" ]; then
  echo "test_inventory: NO PIN at $EXPECTED -- refusing to report a pass" >&2
  exit 2
fi

# ⚠ A pin that examined nothing is the vacuous pass this family exists to
# forbid. Zero tests is never a legitimate state for this repository.
if [ "$COUNT" -eq 0 ]; then
  echo "test_inventory: NOTHING EXAMINED -- zero tests enumerated, refusing to report a pass" >&2
  exit 2
fi

MISSING=$(comm -23 "$EXPECTED" "$ACTUAL")
ADDED=$(comm -13 "$EXPECTED" "$ACTUAL")

echo "test_inventory: enumerated $COUNT tests (pin: $(wc -l < "$EXPECTED" | tr -d ' '))"
if [ -n "$ADDED" ]; then
  echo "test_inventory: NEW tests (allowed; re-pin with --write when convenient):"
  echo "$ADDED" | sed 's/^/    + /'
fi

if [ -n "$MISSING" ]; then
  echo "" >&2
  echo "test_inventory: TESTS DISAPPEARED FROM THE SUITE" >&2
  echo "$MISSING" | sed 's/^/    - /' >&2
  echo "" >&2
  echo "A green suite at a lower total is exactly the failure this pin exists" >&2
  echo "to catch (ENG-0075). If the removal was deliberate and reviewed, say so" >&2
  echo "in the commit and re-pin with:  scripts/ci/test_inventory.sh --write" >&2
  exit 1
fi

echo "test_inventory: PASS -- every pinned test is still present"
exit 0
