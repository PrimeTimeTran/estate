python3 - <<'PY'
mods = {
    131072: "Shift",
    262144: "Control",
    524288: "Option",
    1048576: "Command",
}

for value, name in mods.items():
    print(f"{value:>8} = {value:#010x} = {name}")

print("\nCombinations:")
for a, aname in mods.items():
    for b, bname in mods.items():
        if a < b:
            value = a | b
            print(f"{value:>8} = {value:#010x} = {aname} + {bname}")
PY
