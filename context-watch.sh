#!/usr/bin/env bash

set -euo pipefail

KEY="${1:?usage: $0 <key> <launch-app>}"
LAUNCH_APP="${2:?usage: $0 <key> <launch-app>}"

# Identity for this specific running watcher.
WATCHER_ID="$(uuidgen | cut -c1-8)"

# Number of writes performed by this watcher.
WRITE_COUNT=0

# The actual command used to launch this watcher.
CLI_COMMAND="$0 $*"

# One shared experiment file.
CONTEXT_FILE="host.env.context.json"

# Process identity.
PID=$$
PPID_VALUE=$PPID

mkdir -p "$(dirname "$CONTEXT_FILE")" 2>/dev/null || true

while true; do
	WRITE_COUNT=$((WRITE_COUNT + 1))

	FOCUS="$(
		echo '{
			"command_id": "focused",
			"command": "getFocusedElement",
			"attributes": [
				"AXTitle",
				"AXValue",
				"AXURL"
			]
		}' |
			axorc raw --stdin |
			jq -c '
				select(.data) |
				.data |
				{
					role: (.role // null),
					title: (.attributes.AXTitle.any_value // null),
					value: (.attributes.AXValue.any_value // null),
					url: (.attributes.AXURL.any_value // null),
					textual_content: (.textual_content // null),
					path: (.path // [])
				}
			'
	)"

	# macOS date does not provide GNU date's %N nanoseconds.
	TIMESTAMP="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

	TMP_FILE="${CONTEXT_FILE}.tmp"

	if [[ -f "$CONTEXT_FILE" ]]; then
		cp "$CONTEXT_FILE" "$TMP_FILE"
	else
		echo '{}' > "$TMP_FILE"
	fi

	jq \
		--arg key "$KEY" \
		--arg watcher_id "$WATCHER_ID" \
		--arg launch_app "$LAUNCH_APP" \
		--arg cli_command "$CLI_COMMAND" \
		--arg pid "$PID" \
		--arg ppid "$PPID_VALUE" \
		--arg timestamp "$TIMESTAMP" \
		--argjson writes "$WRITE_COUNT" \
		'
		.[$key] = {
			watcher_id: $watcher_id,
			writes: $writes,
			pid: ($pid | tonumber),
			ppid: ($ppid | tonumber),
			launch_app: $launch_app,
			cli_command: $cli_command,
			last_write_timestamp: $timestamp,
		}
		' \
		"$TMP_FILE" > "${TMP_FILE}.next"

	mv "${TMP_FILE}.next" "$TMP_FILE"
	mv "$TMP_FILE" "$CONTEXT_FILE"

	# Pick a new random interval between 2 and 10 seconds.
	SLEEP_SECONDS=$((RANDOM % 9 + 2))

	echo "[$KEY] watcher=$WATCHER_ID write=$WRITE_COUNT focus=$(jq -r '.role // "-" ' <<< "$FOCUS") sleep=${SLEEP_SECONDS}s"

	sleep "$SLEEP_SECONDS"
done


# jq \
# 		--arg key "$KEY" \
# 		--arg watcher_id "$WATCHER_ID" \
# 		--arg launch_app "$LAUNCH_APP" \
# 		--arg cli_command "$CLI_COMMAND" \
# 		--arg pid "$PID" \
# 		--arg ppid "$PPID_VALUE" \
# 		--arg timestamp "$TIMESTAMP" \
# 		--argjson writes "$WRITE_COUNT" \
# 		--argjson focus "$FOCUS" \
# 		'
# 		.[$key] = {
# 			watcher_id: $watcher_id,
# 			writes: $writes,
# 			pid: ($pid | tonumber),
# 			ppid: ($ppid | tonumber),
# 			launch_app: $launch_app,
# 			cli_command: $cli_command,
# 			last_write_timestamp: $timestamp,
# 			last_write_focus: $focus
# 		}
# 		' \
# 		"$TMP_FILE" > "${TMP_FILE}.next"
