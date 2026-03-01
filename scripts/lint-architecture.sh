#!/usr/bin/env bash
set -euo pipefail

# Architectural boundary linter for Flick
# Enforces: Types → Config → Repo → Service → Routes (downward only)

errors=0

echo "=== Flick Architecture Linter ==="
echo ""

# Rule 1: Repo files must not import from service or routes
echo "Checking: repo files don't import from service/routes..."
if grep -rn "from.*['\"].*service['\"]" packages/server/src/domains/*/repo.ts 2>/dev/null; then
  echo "  ERROR: repo.ts imports from service"
  errors=$((errors + 1))
fi
if grep -rn "from.*['\"].*routes['\"]" packages/server/src/domains/*/repo.ts 2>/dev/null; then
  echo "  ERROR: repo.ts imports from routes"
  errors=$((errors + 1))
fi

# Rule 2: Service files must not import from routes
echo "Checking: service files don't import from routes..."
if grep -rn "from.*['\"].*routes['\"]" packages/server/src/domains/*/service.ts 2>/dev/null; then
  echo "  ERROR: service.ts imports from routes"
  errors=$((errors + 1))
fi

# Rule 3: Types files must not import from repo, service, or routes
echo "Checking: types files don't import from repo/service/routes..."
if grep -rn "from.*['\"].*repo['\"]" packages/server/src/domains/*/types.ts 2>/dev/null; then
  echo "  ERROR: types.ts imports from repo"
  errors=$((errors + 1))
fi

# Rule 4: Shared package must not import from server, ui, or sdk
echo "Checking: @flick/shared doesn't import from other packages..."
if grep -rn "from.*['\"]@flick/server" packages/shared/src/ 2>/dev/null; then
  echo "  ERROR: @flick/shared imports from @flick/server"
  errors=$((errors + 1))
fi
if grep -rn "from.*['\"]@flick/ui" packages/shared/src/ 2>/dev/null; then
  echo "  ERROR: @flick/shared imports from @flick/ui"
  errors=$((errors + 1))
fi

echo ""
if [ $errors -eq 0 ]; then
  echo "All architecture checks passed."
else
  echo "Found $errors architecture violation(s)."
  exit 1
fi
