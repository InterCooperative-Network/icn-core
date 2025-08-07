#!/bin/bash
set -euo pipefail

REPO="InterCooperative-Network/icn-core"
OWNER="InterCooperative-Network"
PROJECT_NAME="ICN Economic Protocol Implementation"
PROJECT_DESC="Implementation tracking for the economic and incentive protocol"

echo "== ICN GitHub Project Script =="

# 0. Ensure correct scopes
echo "Checking authentication scopes..."
gh auth status || { echo "Run 'gh auth login' first!"; exit 1; }
gh auth refresh -s read:project,project,repo

# 1. Create Project Board (if not exists)
echo "Checking for existing project..."
if gh project list --owner $OWNER | grep -q "$PROJECT_NAME"; then
    echo "Project already exists. Skipping creation."
else
    echo "Creating project board..."
    gh project create --owner $OWNER --title "$PROJECT_NAME"
    echo "Created project board: $PROJECT_NAME"
fi

# 2. Create Milestones (no due dates)
function create_milestone() {
    TITLE="$1"
    DESC="$2"
    if gh api "repos/$REPO/milestones" | grep -q "\"title\": \"$TITLE\""; then
        echo "Milestone '$TITLE' exists, skipping."
    else
        echo "Creating milestone '$TITLE'..."
        gh api "repos/$REPO/milestones" -X POST -f title="$TITLE" -f description="$DESC"
    fi
}
create_milestone "M1: Foundation"     "Core economic primitives"
create_milestone "M2: Trust & Governance" "Trust scoring and democratic voting"
create_milestone "M3: CCL Integration"    "Smart contract policies"
create_milestone "M4: Federation"         "Multi-org coordination"
create_milestone "M5: Production"         "Audit and optimization"

# 3. Create Labels
declare -A LABELS
LABELS=(
    ["epic"]="6C40E8"
    ["implementation"]="0E8A16"
    ["copilot-ready"]="1D76DB"
    ["critical"]="D93F0B"
    ["blocked"]="F9D0C4"
    ["milestone-1"]="FBCA04"
    ["milestone-2"]="FBCA04"
    ["milestone-3"]="FBCA04"
    ["milestone-4"]="FBCA04"
    ["milestone-5"]="FBCA04"
)
for LABEL in "${!LABELS[@]}"; do
    COLOR="${LABELS[$LABEL]}"
    if gh label list --repo $REPO | grep -q "^$LABEL"; then
        echo "Label $LABEL exists, skipping."
    else
        echo "Creating label $LABEL..."
        gh label create "$LABEL" --color "$COLOR" --repo $REPO || true
    fi
done

# 4. Create Example Issues (idempotent, checks for existing)
function create_issue() {
    TITLE="$1"
    BODY="$2"
    LABELS="$3"
    MILESTONE="$4"
    if gh issue list --repo $REPO --search "$TITLE" | grep -q "$TITLE"; then
        echo "Issue '$TITLE' already exists. Skipping."
    else
        echo "Creating issue: $TITLE"
        gh issue create --repo $REPO --title "$TITLE" --body "$BODY" --label "$LABELS" --milestone "$MILESTONE"
    fi
}
create_issue \
    "[EPIC] Create icn-economics crate" \
    "Track the creation of the economic primitives crate, including mana, tokens, and trust system." \
    "epic,implementation,critical,milestone-1" \
    "M1: Foundation"
create_issue \
    "[IMPL] Implement ManaLedger" \
    "Implement the core mana ledger with regeneration mechanics.  
Acceptance:  
- Mana regenerates based on compute score  
- Spending reduces balance correctly  
- Cannot spend more than balance  
- Regeneration caps at maximum" \
    "implementation,copilot-ready,milestone-1" \
    "M1: Foundation"
create_issue \
    "[IMPL] Implement TokenLedger" \
    "Implement the token system with support for multiple token classes including non-transferable membership tokens." \
    "implementation,copilot-ready,milestone-1" \
    "M1: Foundation"
create_issue \
    "[IMPL] Add membership credentials to icn-identity" \
    "Extend icn-identity with non-transferable membership credentials." \
    "implementation,copilot-ready,milestone-1" \
    "M1: Foundation"
create_issue \
    "[IMPL] Create compute score calculation" \
    "Implement compute score calculation based on hardware resources, as per protocol." \
    "implementation,copilot-ready,milestone-1" \
    "M1: Foundation"
create_issue \
    "[IMPL] Basic DAG integration for economic history" \
    "Store economic transactions in content-addressable DAG." \
    "implementation,copilot-ready,milestone-1" \
    "M1: Foundation"

echo
echo "All done! Review your project at: https://github.com/$REPO/projects"
echo "You may now add further issues, columns, or automation as needed."