#!/bin/bash
#
# bendy-web-sential Release Script
# Runs CI build → commits version bump → pushes to origin
#

set -e

PROJECT_ROOT="/myproject/rust/bendy-web-sential"
export PATH="$PATH:/root/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin"

cd "$PROJECT_ROOT"

VERSION="${1:-}"
if [ -z "$VERSION" ]; then
    echo "Usage: $0 <version>"
    echo "Example: $0 0.6.0"
    exit 1
fi

echo "=== bendy-web-sential Release v$VERSION ==="
echo ""

# Check for uncommitted changes
if ! git diff-index --quiet HEAD -- 2>/dev/null; then
    echo "ERROR: You have uncommitted changes. Please commit or stash them first."
    exit 1
fi

# Ensure on dev branch
BRANCH=$(git symbolic-ref --short HEAD 2>/dev/null || echo "")
if [ "$BRANCH" != "dev" ]; then
    echo "ERROR: Must be on 'dev' branch. Current: $BRANCH"
    exit 1
fi

# Run CI build
echo "[1/3] Running CI build..."
./scripts/ci-build.sh

# Commit version bump
echo ""
echo "[2/3] Committing version bump to v$VERSION..."
git add -A
git commit -m "chore(release): v$VERSION

Automated release commit.
Built: $(date '+%Y-%m-%d %H:%M:%S')"

# Tag
echo ""
echo "[3/3] Tagging v$VERSION..."
git tag -a "v$VERSION" -m "v$VERSION release"

# Push
echo ""
echo "=== Release Ready ==="
echo "Commits pushed: run 'git push origin dev' and 'git push origin v$VERSION'"
echo ""
echo "Next steps:"
echo "  1. git push origin dev"
echo "  2. git push origin v$VERSION"
echo "  3. Create GitHub release from tag v$VERSION"
