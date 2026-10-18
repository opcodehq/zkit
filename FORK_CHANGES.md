# Maintained changes

- Embed licensed application fonts in the browser platform, repairing missing source-distribution assets.
- Select the bundled font family consistently.
- Provide an owned platform dependency for the Zuse application.
- Retire the browser text handler when focus changes or blurs, and synchronously refresh a missing handler before text insertion. This prevents a newly focused field from editing its predecessor.

The browser application must retain `ApplicationHandle` returned by `run_embedded` for the lifetime of its event loop.
