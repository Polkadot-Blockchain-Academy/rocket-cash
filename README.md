# rocket-cash

![Rocket Cash](./rc-server/static/img/logo-header.png)

🚀 💰 - Web2 digital cash example for PBA smart contracts module

## Run the Server

To start the server locally:

```bash
cd rc-server
cargo run
```

Server will be running locally at: [http://127.0.0.1:8000](http://127.0.0.1:8000)

## Libraries used

- [Rocket - Rust web framework](https://rocket.rs/)
- [rocket_auth - Authentication crate for rocket](https://crates.io/crates/rocket_auth)
- [Diesel - ORM and Query builder for Rust](https://diesel.rs/)
- [Tera - template engine for Rust](https://tera.netlify.app/)
- [Bulma - CSS framework](https://bulma.io/)

## Main Project structure

- `rc-server/migrations` - SQL migration files for Diesel
- `rc-server/static/templates` - Tera templates for pages
- `rc-server/src/main.rs` - URL route handlers
- `rc-server/src/balance.rs` and `rc-server/src/transfer.rs` - ORM/DM Queries

## Example ideas for further development

- Nth caller game - like when a radio station says, the 10th caller wins free tickets.
  Interactions could be hosting a new game, and calling in.
- Lottery - people can purchase tickets and then winner(s) is selected at random to receive the pot from ticket purchases.
- Microblogging - very stripped down version of twitter.
  Interactions could be posting an update and tipping someone when you like their update.
- A sports book - interactions are offering a bet, accepting a bet, and judging an outcome (reporting who really won the sports ball game).

## License

Licensed under the terms of the [GPL-3](./LICENSE.md) or later.
