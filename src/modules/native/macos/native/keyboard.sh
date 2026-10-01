PATTERN='karabiner|skhd|yabai|hammerspoon|jitouch|multitouch|keyboard.?maestro|mac.mouse.fix|estate|eventtap'

echo '=== RUNNING PROCESSES ==='
ps -axo pid,ppid,user,command | grep -Ei "$PATTERN" | grep -v grep

echo '=== USER LAUNCHD SERVICES ==='
launchctl list | grep -Ei "$PATTERN"

echo '=== SYSTEM LAUNCHD SERVICES ==='
sudo launchctl list | grep -Ei "$PATTERN"

echo '=== DISABLED / ENABLED USER SERVICES ==='
launchctl print-disabled "gui/$(id -u)" | grep -Ei "$PATTERN"

echo '=== INSTALLED LAUNCH AGENTS / DAEMONS ==='
find ~/Library/LaunchAgents /Library/LaunchAgents /Library/LaunchDaemons \
  -maxdepth 1 -type f 2>/dev/null | grep -Ei "$PATTERN"

echo '=== SYSTEM EXTENSIONS ==='
systemextensionsctl list

# systemextensionsctl list
# pgrep -afil 'Karabiner'
