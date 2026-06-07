# Brownfield HTML Capture for the Wireframe Gate

A fast way to build the **artifact-specific (UI/web) wireframe** at ④ when the
work is a **brownfield change to an existing web page**: instead of
hand-drawing screens, capture the real rendered page with a browser, edit only
the regions that change, and save a self-contained single file.

## When to use (and when not)

- **Use** when: the work modifies an existing, reachable web surface; the page
  is largely server-rendered or its rendered DOM is stable at capture time. The
  real page becomes the **locked context** and doubles as the ⑤ Static Model
  "current structure" evidence for `03-Architect/05-design.md`.
- **Avoid / fall back** when: heavily hashed/minified class names make region
  edits harder than writing fresh; the page is a JS-heavy SPA whose meaningful
  state only exists after interaction; assets are auth-gated or on cross-origin
  CDNs that block `fetch`. Fall back to: screenshot + DOM outline + a hand
  text-first wireframe.
- **Not for greenfield.** Copying a reference site's full HTML anchors you to
  its design. Brownfield is the sweet spot because you must preserve the
  existing screen anyway.

**Text-first still comes first.** This is an artifact-specific pass, never a
replacement for the ④ text-first wireframe. Group requirements and walk the
journey in text before capturing.

## Recipe

Any way to run JS in the page and read back a string works (DevTools console,
a browser MCP, a headless driver).

1. **Capture baseline.** Open the page in the browser (use the
   already-authenticated profile if the surface needs login). The full rendered
   DOM is `document.documentElement.outerHTML`. This baseline is the **locked
   context** — do not redraw it.
2. **Edit only the changed regions.** Inject the change into the DOM and mark
   it with a visible diff (dashed outline + a small badge), so a cold reader
   can see exactly what changes. Everything you do not touch stays as verified
   reality.
3. **Preserve assets** (only as far as you need): same-origin assets —
   `<img src>`, stylesheet text, `url(...)` inside CSS — can be inlined as
   `data:` URIs via `fetch` + `FileReader.readAsDataURL`. Cross-origin or
   auth-gated fetches that fail should fall back to absolute URLs and be
   recorded as limitations of the capture. Many server-rendered pages use
   system fonts, so the CSS `url()` step is often a no-op.
4. **Save** the edited `outerHTML` as one `.html` file (prepend `<!doctype html>`).
5. **Verify self-containment.** Open the saved file via `file://`, set the
   browser/network to **Offline** when possible, and reload. If it renders
   identically, the single file is server-independent.

## Locked context vs variation point (the ④ contract)

| | meaning | in the wireframe |
|---|---|---|
| **Locked context** | untouched real markup | not a variation point — verified reality |
| **Variation point** | the regions you replaced with mock data | name axis + range (e.g. list 0..N, empty state, overflow, on/off) |

Capturing real HTML does not break "cheap, throwaway instance": you only throw
away the mock in the changed regions; the baseline is real and never redrawn.

## Stack limits (be honest in the wireframe)

④ validates one concrete instance. Record which of these the capture actually
covered, and which are deferred:

| axis | easy case | harder case → note / fall back |
|---|---|---|
| class names | readable utilities (Tailwind) | hashed/minified SPA classes → editing harder |
| fonts | system fonts (0 downloads) | CDN/webfonts → cross-origin `fetch` may fail |
| images | same-origin | auth-gated / external → fetch fails → absolute-URL fallback or placeholder |
| dynamic content | static capture is enough | JS-rendered widgets → frozen at capture-time state |

A page in dev mode may also capture debug overlays (e.g. a debug toolbar);
offline reload usually drops them because their JS does not load — but strip
them explicitly if they remain.
