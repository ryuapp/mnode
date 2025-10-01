# mnode

mnode is a minimal JavaScript runtime for CLI tools. It's built on QuickJS and Rust.

## Install

```sh
cargo install --git https://github.com/ryuapp/mnode mnode
```

## Motivation

Starting with `deno compile`, we've made it easy to distribute JavaScript runtimes and scripts as single executable applications. While similar functionality is possible with Node.js, Bun, or Andromeda, the file sizes are too large for simple CLI tools.\
To solive this issue, We're developing a new JavaScript runtime using QuickJS, the most lightweight JavaScript engine.

Currently, we are only creating the API necessary for [pindeps](https://github.com/ryuapp/pindeps).
