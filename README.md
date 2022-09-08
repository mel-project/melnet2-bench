This repo is a single crate that compiles to two binaries, `mn2-bench-server` and `mn2-bench-client`. `cargo install --path .` in this repo to install both to `$PATH`.

---

`mn2-bench-server` implements a JSON-RPC 2.0 server over TCP (using `nanorpc` and `melnet2`) with two methods:

- `hello_world` which returns the string "hello world"
- `delayed_echo(str: string, secs: number)` which waits for `secs` seconds then returns the content of `str`.

`mn2-bench-server --help` displays detailed help.

---

`mn2-bench-client` spams a massive amount of requests / second to a given `mn2-bench-server`. Currently, it just spams the `hello_world` method.

`mn2-bench-client --help` displays detailed help.
