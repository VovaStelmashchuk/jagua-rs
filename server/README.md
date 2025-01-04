## The server for Jaguar-rs

This is the server for the Jaguar-rs project. It is a simple server that serves the front-end of the project and provides an API using jagua-rs

## How to Run

```bash
cd server
cargo run
```

## How to use

The project has two json file in `testdata` directory. You can use them to test the server.

Good placement file:

```bash
cd server

curl -X POST -H "Content-Type: application/json" -d @testdata/good.json http://0.0.0.0:3030/nest
```

Bad placement file:

```bash
cd server

curl -X POST -H "Content-Type: application/json" -d @testdata/good.json http://0.0.0.0:3030/nest
```

The response json in big, I recomend user `jq` to work with it.
