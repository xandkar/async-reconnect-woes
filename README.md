Attempts to Asyncify Reconnect in a Long Running Client
===============================================================================

The general idea is to have a forever-running client that wants to reuse a
connection with a server as much as possible, despite any interruptions.

More specifically: the client wants to implement a lazy reconnect pattern where
it will periodically poll a server and only connects if the connection is not
already established, re-using existing connection as much as possible, however
if there's an IO error during data transfer - the client drops the connection,
expecting to cause a reconnect on the next iteration.

Pseudocode:

```rust
let mut conn: Option<TcpStream> = None;
loop {
    if conn.is_none() {
        conn = connect().ok();
    }
    if let Some(c) = conn {
        if operation(c).is_err() {
            conn = None;
        }
    }
    sleep(interval);
}
```

Now imagine our client as an actor/worker which also holds some state, so the
client now becomes a `struct`, holding the connection and whatever other state:

```rust
struct Worker {
    conn: Option<TcpStream>,
    state: Whatever,
}

impl Worker {
  ...
}
```

The client wants to perform different operations with the connection (e.g.
receive a message, send different types of messages) at different times and
wants to re-use the above reconnect pattern for all of them.

The internal API for this could be a method (let's call it `with`) which takes
a closure `f` and uses it as the `operation` in the above pseudocode.

See: [`src/bin/client_ok_sync.rs`](src/bin/client_ok_sync.rs).

Usage
-----

### In terminal A

    cargo run --bin server

### In terminal B

    cargo run --bin client_ok_sync

Problem
-------

The basic synchronous version of this works just fine
([`src/bin/client_ok_sync.rs`](src/bin/client_ok_sync.rs)). But when translated
directly to async
([`src/bin/client_err_async.rs.rs`](src/bin/client_err_async.rs.rs)), we
suddenly face a lifetime issue for which I'm yet to find a fully-satisfactory
solution:

```
error: lifetime may not live long enough
  --> src/bin/client_err_async.rs:19:13
   |
16 |         self.with(|stream| {
   |                    ------- return type of closure is tokio::io::util::write_all::WriteAll<'2, tokio::net::TcpStream>
   |                    |
   |                    has type `&'1 mut tokio::net::TcpStream`
...
19 |             stream.write_all(msg)
   |             ^^^^^^^^^^^^^^^^^^^^^ returning this value requires that `'1` must outlive `'2`

error: could not compile `async-reconnect-woes` (bin "client_err_async") due to 1 previous error
```

Solutions
---------

None of these are satisfactory, but do work:

- define `with` as a macro instead of a method:
  [`src/bin/client_ok_async_macro.rs`](src/bin/client_ok_async_macro.rs)
- wrap connection in `Arc<Mutex<..>>`:
  [`src/bin/client_ok_async_arc_mutex.rs`](src/bin/client_ok_async_arc_mutex.rs)
- move the connection to the closure, requiring the closure to return it back
  with the result:
  [`src/bin/client_ok_async_move.rs`](src/bin/client_ok_async_move.rs)
