# Maintained changes

- Embed licensed application fonts in the browser platform, repairing missing source-distribution assets.
- Select the bundled font family consistently.
- Provide an owned platform dependency for the Zuse application.
- Retire the browser text handler when focus changes or blurs, and synchronously refresh a missing handler before text insertion. This prevents a newly focused field from editing its predecessor.
- Refresh handlers for paste and composition as well as typed text. Abandon a composition when its editor loses focus, clear marking on that editor, and ignore late composition events until a new session starts.
- Leave paste shortcuts and IME candidate keys to the browser, so synchronous clipboard actions do not swallow paste and composition confirmation cannot submit a composer.
- Provide asynchronous clipboard reads for explicit Paste actions. Retire delayed paste results after edits, selection/focus changes, read-only transitions or newer requests; preserve native synchronous paste and atomic undo history.
- Include common Meta-key editing bindings in WASM builds for browsers running on macOS.

The browser application must retain `ApplicationHandle` returned by `run_embedded` for the lifetime of its event loop.
