# ARBNCO Proxy

This is a proxy which returns the sensor data from ARBNCO in nicely formatted
way, which allows the HTTP Binding for openHAB to interpret the information.  

This proxy also caches results as not to hit the API call limit.  

A configuration file will automatically be generated in the current working
directory, named `config.toml`.

## Requirements

You'll need to have [Rust] installed on your system. This has been tested with
the nightly build.

You also need `git` to clone this repository.

## Copying and Contributing

This program is distributed under the AGPL 3.0 only license. This means if you
make your own fork of this app, you must release its source to its users under
the same license. You also need to disclose changes done to the software.

The terms can be found in the `LICENSE` file.

## Running and Building

You can simply run it by executing:
```
$ cargo run
```
Or create a release binary by running:
```
$ cargo build --release
```

## Configuration and Usage

Running the proxy will automatically generatae a configuration file
(`config.toml`).

The proxy requires the authentication (API) key for ARBNCO, and the site ID.
All the rest configuration fields are proxy-related fields.

On the first run, it'll automatically quit prompting you to add the
authentication key to the configuration file.

[Rust]: https://rustlang.org
