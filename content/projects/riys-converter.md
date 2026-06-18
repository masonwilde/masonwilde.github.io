---
{
    "title": "RIYS Converter",
    "date": "2025-11-02",
    "description": "A client-side media converter using FFmpeg compiled to WebAssembly. No uploads, no servers."
}
---

A client-side media converter using FFmpeg compiled to WebAssembly. Files never leave the browser - no uploads, no servers.

Supports MP3, WAV, OGG, M4A, FLAC, and AAC output. Drag a file in, pick a format, download the result.

[GitHub](https://github.com/masonwilde/riys-converter)

## Stack

- Vanilla JS + HTML/CSS
- FFmpeg.wasm (`@ffmpeg/ffmpeg`, `@ffmpeg/core-mt`) for transcoding
- Vite for bundling

## How It Works

The app lazy-loads the FFmpeg WASM module on first use. Files are written to FFmpeg's in-memory virtual filesystem, transcoded via standard FFmpeg commands, and the output is triggered as a browser download.

Progress events from FFmpeg drive a progress bar in the UI. After download, the virtual filesystem is cleaned up.

## Technical Details

FFmpeg.wasm requires `SharedArrayBuffer` for multi-threaded WASM workers. This means the page needs specific CORS headers:

```js
// vite.config.js
headers: {
  "Cross-Origin-Opener-Policy": "same-origin",
  "Cross-Origin-Embedder-Policy": "require-corp"
}
```

FFmpeg dependencies are excluded from Vite's pre-bundling to avoid conflicts with WebAssembly module loading.

No framework, no state management. The problem is simple enough that ~140 lines of vanilla JS is the right call.
