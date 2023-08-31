use diesel::{self, prelude::*, result::QueryResult};
use rocket::serde::Serialize;

mod schema {
	table! {
		balances {
			id -> Nullable<Integer>,
			user -> Integer,
			total -> Integer,
		}
	}
}

use self::schema::balances;

use crate::DbConn;

#[derive(Serialize, Queryable, Insertable, Debug, Clone, Default)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = balances)]
pub struct Balance {
	#[serde(skip_deserializing)]
	pub id: Option<i32>,
	pub user: i32,
	pub total: i32,
}

#[derive(Debug, FromForm)]
pub struct Wallet {
	pub user: i32,
	pub total: i32,
}

impl Balance {
	/// Create a new balance for a user
	/// Returns the number of affected rows: 1.
	pub async fn insert(wallet: Wallet, conn: &DbConn) -> QueryResult<usize> {
		conn.run(move |c| {
			let new_balance = Balance { id: None, user: wallet.user, total: wallet.total };
			diesel::insert_into(balances::table).values(&new_balance).execute(c)
		})
		.await
	}

	/// Update Balance for User
	/// Returns the number of affected rows: 1.
	pub async fn update_balance_for_user(
		user: i32,
		new_balance: i32,
		conn: &DbConn,
	) -> QueryResult<usize> {
		conn.run(move |c| {
			let current_balance = diesel::update(balances::table.filter(balances::user.eq(user)));
			current_balance.set(balances::total.eq(new_balance)).execute(c)
		})
		.await
	}

	/// Return a single user's Balance
	pub async fn get_balance_for_user(user: i32, conn: &DbConn) -> QueryResult<Balance> {
		conn.run(move |c| balances::table.filter(balances::user.eq(user)).first::<Balance>(c))
			.await
	}
}
