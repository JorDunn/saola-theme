# Saola Touch Edges

Who owns each screen edge on a touch device, and what a finger on that edge does. Saola
components vendor this file next to the style guide. Like the style guide, it is the
source of truth: if an implementation disagrees with it, the implementation is wrong.

All sizes are logical pixels and are theme tokens (`sizes.*` in `saola-tokens`). Read
them from the theme. Do not copy the numbers into a consumer.

---

## 1. Edge ownership

An edge has **one owner or none**. Two surfaces that both listen on one edge will fight
over the same touch, so no second component may add a handle, a strip or a gesture to
an owned edge.

| Edge | Owner | Notes |
|---|---|---|
| Bottom | saola-launcher (edge strip) | The single owner. saola-keyboard ships **no handle** of its own; the launcher strip's tap opens and closes it (§3). |
| Top | none | |
| Left | none | |
| Right | none | |

The keyboard reserves its exclusive zone **only while it is visible**. While it is
hidden it reserves nothing, so windows reach the bottom edge and the strip sits over
them.

---

## 2. The edge strip

An invisible layer-shell surface along the bottom edge. It draws nothing — no handle,
no pill, no hint line. It only takes touch input.

| Property | Value |
|---|---|
| Thickness | `sizes.edge_strip` (`8px`) |
| Layer | `Overlay` |
| Exclusive zone | `0` — it reserves no space and pushes no window |
| Outputs | All screens (`AllScreens`), one strip per output |
| Owner | saola-launcher (the resident daemon) |

The strip is thin on purpose. It is on the Overlay layer, so it sits above the keyboard
when the keyboard is open, and a thicker strip would eat the keyboard's space bar.

---

## 3. Gestures

Travel is measured within one touch grab, from the touch-down point to the current
point. The two thresholds do not overlap (`tap_tolerance` < `swipe_threshold`, and a
test in `saola-tokens` keeps it that way).

| Gesture | Condition | Action |
|---|---|---|
| Swipe up | Vertical travel upward **≥ `sizes.swipe_threshold`** (`48px`) within the grab | Open the launcher |
| Tap | Total travel **< `sizes.tap_tolerance`** (`10px`) when the touch lifts | Call `io.saola.Keyboard1.Toggle` |
| Anything between | Travel ≥ `tap_tolerance` but upward travel < `swipe_threshold` | **No action.** The touch is dropped. |
| Swipe down | On the open launcher | Dismiss the launcher |
| Tap the scrim | On the open launcher, outside its content | Dismiss the launcher |

The dead band between the two thresholds is deliberate. A short drag is not a sloppy
tap and not a weak swipe; guessing either way opens something the user did not ask
for.

The launcher opens and dismisses on `motion.popover` (style guide §5). There is no
separate launcher motion token.

---

## 4. Interfaces

| Interface | Owner | Spec |
|---|---|---|
| `io.saola.Keyboard1` (`Toggle`) | saola-keyboard | `saola-keyboard/docs/keyboard-spec.md` |
| `Launcher1` | saola-launcher | saola-launcher's spec (Launcher1) |

Paths are relative to the parent directory that holds the Saola repositories. The
D-Bus interfaces are the contract between the strip and the components it drives; this
file only says which gesture calls which method.

The launcher surface itself is in the style guide, §7 "Launcher (grid)".
