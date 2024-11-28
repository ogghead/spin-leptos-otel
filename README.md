# spin-leptos-otel

## OTel manager and dashboard built with Fermyon Spin and Leptos

The dream of this library is to create a flexible-to-distribute Webassembly-based OTel dashboard that can scale to 0.

However, as of now this is not at all production-ready -- getting there will take quite a bit of work to accomplish! Many production-ready open-source OTel dashboards already exist (Grafana/Loki/Prometheus), and I highly recommend using those projects unless you are interested in contributing to this project.

Prequisites:

- Rust [with the `wasm32-wasip1` target](https://www.rust-lang.org/tools/install) - `rustup target add wasm32-wasip1`
- [Spin](https://developer.fermyon.com/spin/v3/install)
- [`cargo-leptos`](https://github.com/leptos-rs/cargo-leptos#getting-started) - `cargo install --locked cargo-leptos`

Build and run:

- `spin up --listen 127.0.0.1:4318 --build` to build and run the server. It will print the application URL.
