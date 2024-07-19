## Overview

`dioxus-chessboard` is a Rust crate designed to provide a web-based chessboard component
for the Dioxus framework. This crate offers an interactive and customizable chessboard
that can be integrated into any web application built with Dioxus.

<p align="center">
  <img src="./example/showcase.gif" />
</p>

## Setup

- Install the latest Dioxus CLI

  As of 19.06.2024, one needs to install the CLI from the Dioxus repo to address the asset
  management [issue](https://github.com/DioxusLabs/dioxus/issues/2641).
  For the same reason, the asset management crate `manganis` is pulled directly from git. Consequently, this crate
  cannot be currently updated on [crates.io](https://crates.io/crates/dioxus-chessboard).

  ```bash
  cargo install --git https://github.com/DioxusLabs/dioxus dioxus-cli
  ```

- Clone this repository
  ```bash
  git clone git@github.com:vnermolaev/dioxus-chessboard.git
  ```   

- Compile the styles
  ``` bash
  npx tailwindcss -i ./input.css -o ./public/tailwind.css --watch
  ```

- Compile the showcase example
  ``` bash
  dx serve --example=showcase --features showcase --hot-reload=true
  ```

- Navigate to [localhost]( http://127.0.0.1:8080)

## Credit

All images are in Public Domain and sourced from [OpenClipart](https://openclipart.org/).