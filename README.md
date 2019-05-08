# aktoro-channel

## Why?

Right now, this crate only provides wrappers around the channel types of
[`futures_channel`](https://rust-lang-nursery.github.io/futures-api-docs/0.3.0-alpha.15/futures_channel/).

Its goal is to provide a public, (hopefully) backward compatible, API
for channels, that can be used by the [`aktoro`](https://github.com/aktoro-rs/)
crates, so that it will be easier to rewrite their internals.
