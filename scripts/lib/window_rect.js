// JXA helper: prints "x,y,w,h" of the largest on-screen window owned by a process name.
// usage: osascript -l JavaScript scripts/lib/window_rect.js <process-name>
ObjC.import('CoreGraphics');
function run(argv) {
  const owner = argv[0];
  const list = ObjC.deepUnwrap(ObjC.castRefToObject($.CGWindowListCopyWindowInfo(0, 0)));
  let best = null, area = 0;
  for (const w of list) {
    if (w.kCGWindowOwnerName !== owner) continue;
    const a = w.kCGWindowBounds.Width * w.kCGWindowBounds.Height;
    if (a > area) { best = w; area = a; }
  }
  if (!best) return '';
  const b = best.kCGWindowBounds;
  return [b.X, b.Y, b.Width, b.Height].join(',');
}
