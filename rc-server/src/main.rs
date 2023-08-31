#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_sync_db_pools;
#[macro_use]
extern crate diesel;

use rocket::{
	fairing::AdHoc,
	form::*,
	fs::{relative, FileServer},
	get, post,
	response::Redirect,
	routes,
	serde::Serialize,
	Build, Rocket, State,
};

use rocket_auth::{prelude::Error, *};

use sqlx::*;

use std::{result::Result, *};

use rocket_dyn_templates::Template;

use crate::{
	balance::{Balance, Wallet},
	transfer::{Transfer, TransferLogItem},
};

pub mod balance;
pub mod transfer;

#[database("sqlite_database")]
pub struct DbConn(diesel::SqliteConnection);

#[derive(FromForm)]
struct NewTransfer {
	receiver: i32,
	amount: i32,
}

#[derive(Debug, Serialize, Default)]
#[serde(crate = "rocket::serde")]
struct Context {
	user: Option<User>,
	balance: Option<Balance>,
	sent: Option<Vec<Transfer>>,
	received: Option<Vec<Transfer>>,
}

#[get("/signup")]
async fn get_signup() -> Template {
	Template::render::<&str, Context>("templates/signup", Default::default())
}

#[post("/signup", data = "<form>")]
async fn post_signup(
	auth: Auth<'_>,
	form: Form<Signup>,
	conn: DbConn,
	users: &State<Users>,
) -> Result<Redirect, Error> {
	auth.signup(&form).await?;
	auth.login(&form.clone().into()).await?;

	let new_user = users.get_by_email(form.email.as_str()).await?;

	Balance::insert(Wallet { user: new_user.id, total: 100 }, &conn).await.ok();

	Ok(Redirect::to("/"))
}

#[get("/login")]
fn get_login() -> Template {
	Template::render::<&str, Context>("templates/login", Default::default())
}

#[post("/login", data = "<form>")]
async fn post_login(auth: Auth<'_>, form: Form<Login>) -> Result<Redirect, Error> {
	let result = auth.login(&form).await;
	result?;
	Ok(Redirect::to("/"))
}

#[get("/logout")]
fn logout(auth: Auth<'_>) -> Result<Template, Error> {
	auth.logout()?;
	Ok(Template::render::<&str, Context>("templates/logout", Default::default()))
}

#[get("/user")]
async fn user(user: User, conn: DbConn) -> Result<Template, Error> {
	let balance = Balance::get_balance_for_user(user.id, &conn).await.ok();
	let sent: Option<Vec<Transfer>> = Transfer::all_sent_by(user.id, &conn).await.ok();
	let received: Option<Vec<Transfer>> = Transfer::all_received_by(user.id, &conn).await.ok();

	Ok(Template::render::<&str, Context>(
		"templates/user",
		Context { user: Some(user), balance, sent, received },
	))
}

#[post("/transfer", data = "<form>")]
async fn make_transfer(
	user: User,
	form: Form<NewTransfer>,
	conn: DbConn,
	users: &State<Users>,
) -> Result<Redirect, &'static str> {
	if form.amount < 0 {
		return Err("You can not transfer negative cash")
	}

	if form.receiver == user.id {
		return Err("You can not transfer to yourself")
	}

	match users.get_by_id(form.receiver).await {
		Err(_e) => return Err("Receiving user does not exist"),
		_ => (),
	}

	let sender_balance = Balance::get_balance_for_user(user.id, &conn)
		.await
		.unwrap_or(Default::default());

	let receiver_balance = Balance::get_balance_for_user(form.receiver, &conn)
		.await
		.unwrap_or(Default::default());

	if form.amount > sender_balance.total {
		return Err("Insufficient balance")
	}

	let sender_new_balance = sender_balance.total - form.amount;
	let receiver_new_balance = receiver_balance.total + form.amount;

	Balance::update_balance_for_user(user.id, sender_new_balance, &conn).await.ok();

	Balance::update_balance_for_user(form.receiver, receiver_new_balance, &conn)
		.await
		.ok();

	Transfer::insert(
		TransferLogItem { sender: user.id, receiver: form.receiver, amount: form.amount },
		&conn,
	)
	.await
	.ok();

	Ok(Redirect::to("/user"))
}

#[get("/")]
async fn index(user: Option<User>) -> Redirect {
	match user {
		Some(_u) => Redirect::to("/user"),
		_ => Redirect::to("/login"),
	}
}

async fn run_migrations(rocket: Rocket<Build>) -> Rocket<Build> {
	use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

	const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

	DbConn::get_one(&rocket)
		.await
		.expect("database connection")
		.run(|conn| {
			conn.run_pending_migrations(MIGRATIONS).expect("diesel migrations");
		})
		.await;

	rocket
}

#[tokio::main]
async fn main() -> Result<(), Error> {
	let db_path = std::path::Path::new("db/db.sqlite");

	if !db_path.exists() {
		println!("No db detected. Initializing new db in {:?}.", db_path);
		std::fs::File::create(db_path).expect("Could not create database file");
	}

	let conn = SqlitePool::connect("db/db.sqlite").await?;
	let users: Users = conn.clone().into();
	users.create_table().await?;

	let _ = rocket::build()
		.attach(DbConn::fairing())
		.attach(Template::fairing())
		.attach(AdHoc::on_ignite("Run Migrations", run_migrations))
		.manage(conn)
		.manage(users)
		.mount("/", FileServer::from(relative!("static")))
		.mount(
			"/",
			routes![
				index,
				user,
				make_transfer,
				get_signup,
				post_signup,
				get_login,
				post_login,
				logout
			],
		)
		.launch()
		.await
		.unwrap();

	Ok(())
}
