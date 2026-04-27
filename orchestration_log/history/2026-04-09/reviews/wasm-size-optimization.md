# WASM Size Optimization — Investigation Report

Date: 2026-04-09

## The Question
Is the 1.87MB release WASM (471KB brotli) at its optimization floor, or is there room?

## Current Config
- `opt-level = 'z'`, `lto = true`, `codegen-units = 1` (wasm-release profile)
- wasm-opt `-Oz` applied automatically by cargo-leptos 0.3.2
- Brotli pre-compression at quality 11 (best)

## Findings

### twiggy profiling
- Binary is well-distributed across ~8,000 code/data items
- All symbols stripped — no named functions visible
- 81.60% dominated by function table
- No single dominant bloat source (no 500KB regex blob)
- LTO + wasm-opt effectively eliminate dead code

### opt-level comparison
| | 'z' | 's' |
|---|---|---|
| Raw | 1.87 MB | 2.02 MB |
| Brotli | 471 KB | 504 KB |
| Difference | — | +7.9% larger |

**Verdict:** 'z' is optimal. Keep it.

### regex-in-WASM
- `leptos_config` pulls `regex v1.12.3` into WASM dependency tree
- twiggy shows no 500KB blob → LTO+wasm-opt eliminate the dead code
- A `[patch.crates-io]` attempt to remove regex broke `leptos_config`'s Cargo.toml parsing (reverted)
- **Conclusion:** Not worth patching locally. File upstream issue when ready.

### Remaining optimization headroom
| Technique | Estimated savings | Effort | Risk | Status |
|-----------|------------------|--------|------|--------|
| `build-std` (nightly) | 10-30% | Medium | Medium (nightly dep) | Not applied |
| Code splitting (`--split`) | Initial load only | Medium | Low-Medium | Not applied |
| wasm-opt | Already applied | — | — | Done |
| Brotli compression | Already applied | — | — | Done |

### Production size assessment
471KB brotli is reasonable for a full-stack Leptos app. Minimal Leptos hello-world is ~200-300KB brotli. Framework overhead (reactive system, router, server function client) accounts for the base cost.

## Sources
- Leptos Book: https://book.leptos.dev/deployment/binary_size.html
- Rust WASM Book: https://rustwasm.github.io/docs/book/reference/code-size.html
- cargo-leptos issue #441 (wasm-opt + strip interaction)
- cargo-leptos issue #501 (aggressive optimizations)
