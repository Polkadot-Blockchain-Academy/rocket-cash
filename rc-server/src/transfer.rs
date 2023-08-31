use chrono::prelude::{DateTime, Local};
use diesel::{self, prelude::*, result::QueryResult};
use rocket::serde::Serialize;

mod schema {
	table! {
		transfers {
			id -> Nullable<Integer>,
			sender -> Integer,
			receiver -> Integer,
			amount -> Integer,
			transferred_on -> Text
		}
	}
}

use self::schema::transfers;

use crate::DbConn;

#[derive(Serialize, Queryable, Insertable, Debug, Clone)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = transfers)]
pub struct Transfer {
	#[serde(skip_deserializing)]
	pub id: Option<i32>,
	pub sender: i32,
	pub receiver: i32,
	pub amount: i32,
	pub transferred_on: String,
}

#[derive(Debug)]
pub struct TransferLogItem {
	pub sender: i32,
	pub receiver: i32,
	pub amount: i32,
}

impl Transfer {
	/// Return Vec of all transfers by sender
	pub async fn all_sent_by(sender: i32, conn: &DbConn) -> QueryResult<Vec<Transfer>> {
		conn.run(move |c| {
			transfers::table
				.filter(transfers::sender.eq(sender))
				.order(transfers::transferred_on.desc())
				.load::<Transfer>(c)
		})
		.await
	}

	/// Return Vec of all transfers to recipient
	pub async fn all_received_by(receiver: i32, conn: &DbConn) -> QueryResult<Vec<Transfer>> {
		conn.run(move |c| {
			transfers::table
				.filter(transfers::receiver.eq(receiver))
				.order(transfers::transferred_on.desc())
				.load::<Transfer>(c)
		})
		.await
	}

	/// Record a new balance transfer into the log
	/// Returns the number of affected rows: 1.
	pub async fn insert(transfer_log_item: TransferLogItem, conn: &DbConn) -> QueryResult<usize> {
		conn.run(move |c| {
			let transferred_on: DateTime<Local> = Local::now();
			let new_transfer = Transfer {
				id: None,
				sender: transfer_log_item.sender,
				receiver: transfer_log_item.receiver,
				amount: transfer_log_item.amount,
				transferred_on: transferred_on.to_rfc3339(),
			};
			diesel::insert_into(transfers::table).values(&new_transfer).execute(c)
		})
		.await
	}
}
