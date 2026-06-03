# Examples package

## basic
A basic example showing the standard usage of the library.

## mem_leak
This example is to prove that there are no memory leaks; a finished task must free the allocated memory.

#### MACOS

Build and run the target with the 'leaks' program.
```console
cargo build --example mem_leak
leaks -atExit -- ./target/debug/examples/mem_leak
```

Output example
```console
Process:         mem_leak [2683]
Path:            /Users/USER/*/mem_leak
Load Address:    0x104e34000
Identifier:      mem_leak
Version:         0
Code Type:       ARM64
Platform:        macOS
Parent Process:  leaks [2682]
Target Type:     live task

Date/Time:       2026-06-03 11:25:18.117 +0200
Launch Time:     2026-06-03 11:25:17.604 +0200
OS Version:      macOS 26.4 (25E246)
Report Version:  7
Analysis Tool:   /usr/bin/leaks

Physical footprint:         4769K
Physical footprint (peak):  4769K
Idle exit:                  untracked
----

leaks Report Version: 4.0, multi-line stacks
Process 2683: 192 nodes malloced for 12 KB
Process 2683: 0 leaks for 0 total leaked bytes.
```
