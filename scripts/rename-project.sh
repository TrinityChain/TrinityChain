#!/bin/bash
# rename-project.sh - Systematically rename all project references
# Usage: ./rename-project.sh <old_name> <new_name>

set -e

if [ "$#" -ne 2 ]; then
    echo "Usage: $0 <old_name> <new_name>"
    echo "Example: $0 siertrichain trinitychain"
    exit 1
fi

OLD_NAME=$1
NEW_NAME=$2

# Convert to different cases
OLD_LOWER=$(echo "$OLD_NAME" | tr '[:upper:]' '[:lower:]')
NEW_LOWER=$(echo "$NEW_NAME" | tr '[:upper:]' '[:lower:]')
OLD_TITLE=$(echo "$OLD_NAME" | sed 's/.*/\u&/')
NEW_TITLE=$(echo "$NEW_NAME" | sed 's/.*/\u&/')

echo "🔄 Renaming project from $OLD_NAME to $NEW_NAME"
echo ""

# 1. Update Cargo.toml
echo "📦 Updating Cargo.toml..."
sed -i "s/name = \"$OLD_LOWER\"/name = \"$NEW_LOWER\"/g" Cargo.toml

# 2. Update all Rust source files
echo "🦀 Updating Rust source files..."
find . -name "*.rs" -type f -exec sed -i "s/use $OLD_LOWER::/use $NEW_LOWER::/g" {} +
find . -name "*.rs" -type f -exec sed -i "s/$OLD_LOWER::/$NEW_LOWER::/g" {} +
find . -name "*.rs" -type f -exec sed -i "s/\.$OLD_LOWER/.$NEW_LOWER/g" {} +
find . -name "*.rs" -type f -exec sed -i "s/$OLD_LOWER\.db/$NEW_LOWER.db/g" {} +
find . -name "*.rs" -type f -exec sed -i "s/\"$OLD_LOWER/\"$NEW_LOWER/g" {} +
find . -name "*.rs" -type f -exec sed -i "s/$OLD_TITLE/$NEW_TITLE/g" {} +

# 3. Update all markdown files
echo "📝 Updating documentation..."
find . -name "*.md" -type f -exec sed -i "s/$OLD_TITLE/$NEW_TITLE/g" {} +
find . -name "*.md" -type f -exec sed -i "s/$OLD_LOWER/$NEW_LOWER/g" {} +

# 4. Rename database if it exists
if [ -f "$OLD_LOWER.db" ]; then
    echo "💾 Renaming database..."
    mv "$OLD_LOWER.db" "$NEW_LOWER.db"
fi

# 5. Update wallet directory references
echo "👛 Updating wallet directory references..."
find . -name "*.rs" -type f -exec sed -i "s/\\.${OLD_LOWER}/.$NEW_LOWER/g" {} +

echo ""
echo "✅ Rename complete!"
echo ""
echo "Next steps:"
echo "  1. Review changes: git status"
echo "  2. Build project: cargo build --release"
echo "  3. Run tests: cargo test"
echo "  4. Commit changes: git add . && git commit -m 'Rename project to $NEW_TITLE'"
