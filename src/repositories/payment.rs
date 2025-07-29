use chrono::Utc;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use std::io;

use crate::models::payment::PaymentData;

pub struct PaymentRepository {
    pool: Pool<SqliteConnectionManager>,
}

impl PaymentRepository {
    pub fn new(pool: Pool<SqliteConnectionManager>) -> Self {
        Self { pool }
    }

    pub fn save_payment(&self, payment_data: &PaymentData) -> io::Result<()> {
        let conn = self
            .pool
            .get()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        conn.execute(
            "INSERT INTO payments (correlation_id, amount, processor, processed_at) VALUES (?1, ?2, ?3, ?4)",
            (
                payment_data.correlation_id.as_bytes(),
                (&payment_data.amount * Decimal::from(100)).to_i64().unwrap(),
                payment_data.is_default_processor,
                Utc::now().timestamp_millis(),
            ),
        )
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        Ok(())
    }

    //GET /payments-summary?from=2020-07-10T12:34:56.000Z&to=2020-07-10T12:35:56.000Z
    pub fn get_payments_summary(&self) -> io::Result<Vec<(bool, i64, i64)>> {
      // Falta adicionar: 
      // - Filtros de data (from, to)

      // Falta comunicar com o outro microserviço para obter o total de pagamentos

        let conn = self
            .pool
            .get()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        let mut stmt = conn
            .prepare("SELECT processor, COUNT(*), SUM(amount) FROM payments GROUP BY processor")
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        let rows = stmt
            .query_map([], |row| {
                let is_default: bool = row.get(0)?; // Lê bool diretamente
                let total_requests: i64 = row.get(1)?;
                let total_amount_cents: i64 = row.get(2)?;
                Ok((is_default, total_requests, total_amount_cents))
            })
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?);
        }

        Ok(results)
    }
}
