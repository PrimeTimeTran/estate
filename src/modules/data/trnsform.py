python3 - <<'PY'
import json
import re
from pathlib import Path

src = Path("old-apple-symbolic-hotkeys.md")
dst = Path("apple-symbolic-hotkeys.json")

text = src.read_text()

# Matches:
# # Command name - Default Shortcut
#
# 7 : { enabled : 1, value : { parameters : ( ... ), type : standard, }, },
comment_re = re.compile(
    r"#\s*(.*?)\s*(?:-\s*(.*?))?\s*$"
)

entry_re = re.compile(
    r"(?m)^\s*(\d+)\s*:\s*\{\s*"
    r"enabled\s*:\s*(\d+)"
    r"(?:\s*,\s*value\s*:\s*\{\s*"
    r"parameters\s*:\s*\(\s*([^)]*?)\s*\)"
    r"\s*,\s*type\s*:\s*([^,\s}]+)"
    r"\s*,?\s*\})?"
    r"\s*,?\s*\}\s*,?"
)

lines = text.splitlines()

result = {"AppleSymbolicHotKeys": {}}

pending_comment = None
pending_shortcut = None

for line in lines:
    line = line.strip()

    if not line:
        continue

    # Capture the comment belonging to the next entry.
    if line.startswith("#"):
        m = comment_re.match(line)
        if m:
            pending_comment = m.group(1).strip()
            pending_shortcut = m.group(2).strip() if m.group(2) else None
        continue

    m = entry_re.match(line)
    if not m:
        continue

    hotkey_id = m.group(1)
    enabled = bool(int(m.group(2)))

    entry = {
        "name": pending_comment,
        "default": pending_shortcut,
        "enabled": enabled,
    }

    parameters = m.group(3)
    value_type = m.group(4)

    if parameters is not None:
        params = [
            int(x.strip())
            for x in parameters.split(",")
            if x.strip()
        ]

        entry["value"] = {
            "type": value_type,
            "parameters": params,
        }

    result["AppleSymbolicHotKeys"][hotkey_id] = entry

    pending_comment = None
    pending_shortcut = None

dst.write_text(
    json.dumps(result, indent=2, ensure_ascii=False) + "\n"
)

print(f"wrote {dst}")
print(f"entries: {len(result['AppleSymbolicHotKeys'])}")
PY