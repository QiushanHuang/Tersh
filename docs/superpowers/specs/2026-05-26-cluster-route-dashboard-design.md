# Cluster Route Dashboard Design

Date: 2026-05-26

## Goal

Make `tersh --c` visually communicate how a selected server is reached while keeping the multi-host health overview useful. The interface should feel more professional by combining a route map with compact resource bars.

## Scope

This change is limited to the cluster status TUI.

In scope:

- Add a selected-host route visualization.
- Add terminal-friendly resource bars for memory, storage, and GPU when bounded utilization data is available.
- Preserve CPU load as text, because the current probe reports load average rather than CPU utilization.
- Preserve the existing host list, detail view, footer keys, help overlay, and `s`/`t` behavior.
- Keep rendering robust on wide, medium, and narrow terminals.

Out of scope:

- Changing SSH probe behavior.
- Adding a new inventory format.
- Adding mouse support.
- Adding a third top-level mode.
- Replacing the current two-column mental model.

## Product Direction

The selected host should answer two questions quickly:

- How do I reach this machine?
- Is this machine healthy enough to use?

The route map answers the first question. Resource bars answer the second. The host list remains the navigation anchor, because users still need fast movement across machines.

## Layout

Wide layout, 100 columns and above:

- Left: host list.
- Right: vertical split containing:
  - `Route`: selected-host connection path.
  - `Detail`: host metadata, resource bars, recent log.

Medium layout, 72 to 99 columns:

- Top: host list.
- Bottom: selected-host dashboard containing route, details, resource bars, and log.

Narrow layout, below 72 columns:

- Default remains host list only.
- `l` opens the existing detail mode.
- Detail mode shows route and metrics in a compact single-column layout.

## Route Visualization

Route display is selected-host driven.

For local hosts:

```text
LOCAL ONLY
local alias
```

For direct remote hosts:

```text
LOCAL => SERVER
user@host
```

For remote hosts with a jump host:

```text
LOCAL => JUMP => SERVER
jump-target => ssh-target
```

The route panel also shows the exact target command shape in compact form:

- Direct: `ssh user@host`
- Jump: `ssh -J jump-target user@host`
- Local: the configured local shell or local Tersh target is implied, not expanded.

The route panel must not require Unicode line drawing. ASCII arrows are enough and safer for remote terminals.

## Resource Bars

Resource bars should be derived at render time from existing `ProbeReport` strings. The data model remains unchanged unless implementation proves a small pure helper type is clearer.

Initial bar rules:

- Memory: parse the first percentage token, e.g. `512/1024 MB (50%)`. If the string describes free memory, e.g. `42% free`, invert it for the bar and label the bar as used while preserving the raw text.
- Storage: parse the first percentage token, e.g. `8G/20G 40% used`.
- GPU: if the string is `none`, show an empty neutral bar; otherwise parse the first percentage if present and show the raw string.
- CPU load: show the raw load average text only. Do not convert it to a utilization bar without probing CPU core count and utilization.

If parsing fails, show the raw text with an empty neutral bar. Rendering must never fail because a probe string is unusual.

## Status Color Rules

- Online host and healthy metric bars: green.
- Checking or stale state: yellow.
- Offline, timeout, or auth failure: red.
- Unknown or unparsable metrics: gray.
- The selected row keeps the existing high-contrast cyan background.

Colors are enhancements, not requirements. Text labels must still convey state.

## Error Handling

The route and metric helpers are pure render helpers:

- No filesystem access.
- No SSH calls.
- No panics for empty strings, malformed percentages, missing jump targets, or tiny rectangles.
- Existing last-good stale behavior remains unchanged.

## Testing

Add render tests that assert:

- Wide layout contains `Route`, route arrows, selected host, resource bar labels, text CPU load, and footer keys.
- Jump-host route renders `LOCAL`, `JUMP`, `SERVER`, `ssh -J`, and the proxy jump target.
- Direct/local route cases render without `JUMP`.
- Narrow detail mode includes route information and resource labels.
- Resource bars preserve raw metric text when percentages are parseable or absent.
- CPU load remains text-only.

Existing cluster behavior tests should remain valid.
