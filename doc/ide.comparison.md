# IDE Comparisons

## Base Memory Utilization

- VSCode: 150mb
- Zed: 330mb
- Rust Rover: 4.25gb

ps -Ax -o pid,rss,comm | grep -E "RustRover|Code|Zed"

```sh
for app in "RustRover" "Code" "Zed"; do
    total_kb=$(ps -Ax -o rss,comm | grep -i "$app" | awk '{sum+=$1} END {print sum}')
    if [ ! -z "$total_kb" ] && [ "$total_kb" -gt 0 ]; then
        total_mb=$(echo "scale=2; $total_kb / 1024" | bc)
        echo "📊 Total Memory Footprint for $app: ${total_mb} MB"
    else
        echo "❌ $app does not appear to be running."
    fi
done
```

This output is gold. It perfectly illustrates exactly how multi-process vs. single-process applications hide their actual memory footprints.
When you sum up the raw Kilobyte numbers (RSS) from your terminal output and convert them to Megabytes, the real picture changes dramatically:

- VS Code (Total): 1,394.25 MB (~1.4 GB)
- RustRover (Total): 860.77 MB (~860 MB)
- Zed (Total): 107.80 MB (~108 MB)

---

## What to Learn from Your Data

### 1. VS Code Lies to You in Activity Monitor

If you open Activity Monitor and look only at the main entry named "Visual Studio Code" (PID 94339), it only reports 164.6 MB.
However, look at all the Code Helper processes it spawned:

- PID 94345 (Renderer): 432.6 MB (Handling your visual UI)
- PID 2137 (Plugin): 233.5 MB (Running an extension background server)
- PID 94427 (Helper): 134.2 MB

When you add up all 17 independent processes running for VS Code, it is actually using 1.4 GB of RAM right now, not 150 MB!

### 2. RustRover is Natively Consuming Less Than 1 GB Here

The primary rustrover instance (PID 73411) is taking up 691.9 MB. Your Activity Monitor might say it is using 4 GB because it includes the Virtual Memory space the JVM pre-allocated from the OS, but its active, physical footprint in your actual RAM is only 860 MB.
(Note: It also has full-line-inference and embeddings-server running, which are JetBrains' local AI autocomplete features).

### 3. Zed is the Undisputed Efficiency King

Written entirely in native Rust, Zed is running practically naked.
Its primary process (PID 88309) takes just 106.5 MB, and it only spawns one tiny extension server (color-lsp). It genuinely lives up to the lightweight hype.
------------------------------

## How to Move Forward with Benchmarking

This data shows you have all three IDEs running concurrently. To make this a flawless science experiment, you should calculate their Idle vs. Active Load profiles:

1.  Test the "Active" Footprint: Open a massive, identical Rust project in each IDE one by one. Run a cargo build inside the IDE terminal and perform a global search (Shift+Shift or Cmd+Shift+F).
2.  Rerun Your Command: Run your ps query again during that heavy workload. You will likely watch VS Code's helper processes balloons past 2.5 GB as rust-analyzer fires up, while RustRover will stay stably locked to its pre-allocated JVM size.

Would you like a specialized Zsh shell alias you can drop into your .zshrc file so you can type a quick shortcut (like ide-ram) to print this aggregated chart anytime?



### Ideas

Outline panel should support multiple ways of searching
- filename (string literal)
- vim nav
- 'category nav'. Imagine moving jumping focusing between enums, structs, impls, functions or files/folders inside of tree view
  - "next file down", "next folder up"
- Also there should be ways of hiding 'all of type' or 'showing all of type' in a file
- also a 'secondary config menu'