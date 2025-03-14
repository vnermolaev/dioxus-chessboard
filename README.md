## Overview

`dioxus-chessboard` is a Rust crate designed to provide a web-based chessboard component
for the Dioxus framework. This crate offers an interactive and customizable chessboard
that can be integrated into any web application built with Dioxus.

<p align="center">
  <img src="./example/showcase.gif" />
</p>

## Setup

- Install the latest Dioxus CLI

  ```bash
  cargo install dioxus-cli
  ```
- Clone this repository
  ```bash
  git clone git@github.com:vnermolaev/dioxus-chessboard.git
  ```   

- Change to the example directory and execute all the commands from there
  ```bash
  cd ./example
  ```

- Install the dependencies to compile Tailwind CSS.

  ``` bash
  npm i
  ```

- Compile the styles
  ``` bash
  npm run buildcss
  ```

- Compile the showcase example
  ``` bash
  dx serve --example=showcase --features showcase --hot-reload=true
  ```

- Navigate to [localhost](http://127.0.0.1:8080)

## Credit

All images are in Public Domain and sourced from [OpenClipart](https://openclipart.org/).