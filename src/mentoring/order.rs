// Copyright 2023. The resback authors all rights reserved.

use chrono::{DateTime, Utc};
use sqlx::MySql;

use crate::{
    schema::{MentoringOrderListSchema, MentoringOrderSchema},
    user::account::UserId,
    Result,
};

use super::{
    schedule::{MentoringMethod, MentoringTime},
    MentoringMethodKind,
};

#[derive(sqlx::FromRow, Debug)]
struct MentoringOrderRow {
    id: u64,
    buyer_id: UserId,
    seller_id: Option<UserId>,
    time_id: u64,
    #[sqlx(rename = "method_id")]
    method_kind: MentoringMethodKind,
    price: u32,
    content: String,
    created_at: DateTime<Utc>,
}

pub struct MentoringOrder {
    id: u64,
    buyer_id: UserId,
    seller_id: Option<UserId>,
    time: MentoringTime,
    method: MentoringMethod,
    price: u32,
    content: String,
    created_at: DateTime<Utc>,
}

impl MentoringOrder {
    pub async fn from_id(id: u64, pool: &sqlx::Pool<MySql>) -> Result<Self> {
        let row = sqlx::query_as!(
            MentoringOrderRow,
            "SELECT
id,
buyer_id as `buyer_id: UserId`,
seller_id as `seller_id: UserId`,
time_id as `time_id: u64`,
method_id as method_kind,
price,
content,
created_at FROM mentoring_order WHERE id = ?",
            id
        )
        .fetch_one(pool)
        .await?;

        Self::from_row(&row, pool).await
    }

    pub async fn from_buyer_id(buyer_id: UserId, pool: &sqlx::Pool<MySql>) -> Result<Vec<Self>> {
        let rows = sqlx::query_as!(
            MentoringOrderRow,
            "SELECT
id,
buyer_id as `buyer_id: UserId`,
seller_id as `seller_id: UserId`,
time_id,
method_id as method_kind,
price,
content,
created_at FROM mentoring_order WHERE buyer_id = ?",
            buyer_id
        )
        .fetch_all(pool)
        .await?;

        Self::from_rows(&rows, pool).await
    }

    pub async fn from_seller_id(seller_id: UserId, pool: &sqlx::Pool<MySql>) -> Result<Vec<Self>> {
        let rows = sqlx::query_as!(
            MentoringOrderRow,
            "SELECT
id,
buyer_id as `buyer_id: UserId`,
seller_id as `seller_id: UserId`,
time_id,
method_id as method_kind,
price,
content,
created_at FROM mentoring_order WHERE seller_id = ?",
            seller_id
        )
        .fetch_all(pool)
        .await?;

        Self::from_rows(&rows, pool).await
    }

    pub async fn create(
        buyer_id: UserId,
        seller_id: UserId,
        time: &MentoringTime,
        method: &MentoringMethod,
        price: u32,
        content: &str,
        pool: &sqlx::Pool<MySql>,
    ) -> Result<Self> {
        let id = sqlx::query!(
            "INSERT INTO mentoring_order (
buyer_id,
seller_id,
time_id,
method_id,
price,
content) VALUES (?, ?, ?, ?, ?, ?)",
            buyer_id,
            seller_id,
            time.id(),
            method.kind(),
            price,
            content
        )
        .execute(pool)
        .await
        .map(|result| result.last_insert_id())?;

        MentoringOrder::from_id(id, pool).await
    }

    pub fn buyer_id(&self) -> UserId {
        self.buyer_id
    }

    pub fn seller_id(&self) -> Option<UserId> {
        self.seller_id
    }

    async fn from_row(row: &MentoringOrderRow, pool: &sqlx::Pool<MySql>) -> Result<Self> {
        Ok(Self {
            id: row.id,
            buyer_id: row.buyer_id,
            seller_id: row.seller_id,
            time: { MentoringTime::from_id(row.time_id, pool).await? },
            method: { MentoringMethod::from_kind(row.method_kind, pool).await? },
            price: row.price,
            content: row.content.clone(),
            created_at: row.created_at,
        })
    }

    async fn from_rows(
        rows: &Vec<MentoringOrderRow>,
        pool: &sqlx::Pool<MySql>,
    ) -> Result<Vec<Self>> {
        let mut orders = Vec::<Self>::new();

        for row in rows {
            orders.push(Self::from_row(row, pool).await?)
        }

        Ok(orders)
    }
}

impl From<MentoringOrder> for MentoringOrderSchema {
    fn from(value: MentoringOrder) -> Self {
        Self {
            id: value.id,
            buyer_id: value.buyer_id,
            seller_id: value.seller_id,
            time: value.time.hour(),
            method: value.method.kind(),
            price: value.price,
            content: value.content,
            created_at: value.created_at,
        }
    }
}

impl From<Vec<MentoringOrder>> for MentoringOrderListSchema {
    fn from(value: Vec<MentoringOrder>) -> Self {
        Self { orders: { value.into_iter().map(|order| order.into()).collect() } }
    }
}
