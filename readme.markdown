# Boozle

**This project is in the early stages of development. It is not currently usable, and may not even compile.**

Boozle is a RPC framework designed to feel good to use. It aims to be intuitive, expressive, and effortless. Boozle's long-term goal is to be cross-language,
but currently it only supports Rust. With this goal in mind, RPC protocols are defined in a "rusty" DSL with the extension `.boozle`, rather than using Rust macros. 

This project was inspired by years of working in various RPC frameworks and always left feeling not quite satisfied. In particular:
  - Many generators do not generate idiomatic code for the target language (e.g., gRPC C++). Calling a method on a remote object should feel like calling a method on a local one.
  - Many RPC frameworks are closely coupled to their transport (e.g., gRPC). What if I want to talk to a device over USB?
  - Many RPC frameworks encourage a strict client-server design. What if I want several nodes to seamlessly work together?
  - No Cap'n Proto implementation implements Level 3 or 4 of their [RPC specification](https://capnproto.org/rpc.html).


Boozle has several features (some unique):
  - **Live Objects**: Objects passed between nodes are "live". Unlike gRPC or ROS, types have (and can only have) methods. To avoid round trips for trivial getters, methods can be annotated as `cached`.
  - **Meshing**: Objects may be proxied between multiple nodes. This means that node A can receive an object from node B that originated on node C. Three-way introduction (where possible) is also underway.
  - **Expression-based Execution Architecture**: Avoids round-trips for pipelined RPC operations (e.g., `object.do_thing(using.this())` is executed as a single expression).
  - **Rich Type System**: RPC protocol types can have generic type arguments.
  - **Streams**: Streams (through the `Source<T>` and `Sink<T>` types) can be both a parameter and return value on a RPC method.
  - **Transport Agnostic**: Boozle can work over any transport that is reliable (unordered is okay).
  - **Upgradable Protocol Definitions**: Extend an existing RPC protocol without breaking older nodes.
  - **Designed for Parallelism**: Boozle is completely async and lock-free by design.

Immediate goals (in rough order of priority):
  - Complete Rust implementation
  - Examples
  - Documentation
  - Tests
  - CI/CD and other GitHub / DevOps QoL integrations

Future goals (in rough order of priority):
  - Bindings / Implementations for other languages (C++, JavaScript, Python, etc.)
  - Establishing a direct connections between nodes to alleviate the overhead of proxying objects
  - Automatically generate idiomatic boozle bindings from other RPC protocol definitions (gRPC, ROS, etc.). Boozle all the RPC!
