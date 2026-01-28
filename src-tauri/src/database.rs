use rusqlite::{Connection, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Debug, Serialize, Deserialize)]
pub struct Transaction {
    pub id: i64,
    pub amount: i64, // Stored in cents
    pub description: String,
    pub category: String,
    pub date: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewTransaction {
    pub amount: i64,
    pub description: Option<String>,
    pub category: Option<String>,
}

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn new(db_path: PathBuf) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS transactions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                amount INTEGER NOT NULL,
                description TEXT NOT NULL,
                category TEXT NOT NULL,
                date TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS categories (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                is_default INTEGER NOT NULL DEFAULT 0
            )",
            [],
        )?;

        let count: i64 = conn.query_row("SELECT COUNT(*) FROM categories", [], |row| row.get(0))?;
        if count == 0 {
            let defaults = vec![
                "Food & Dining",
                "Transportation",
                "Shopping",
                "Entertainment",
                "Bills & Utilities",
                "Healthcare",
                "Income",
                "Other"
            ];
            for category in defaults {
                conn.execute(
                    "INSERT INTO categories (name, is_default) VALUES (?1, 1)",
                    [category],
                )?;
            }
        }

        Ok(Database {
            conn: Mutex::new(conn),
        })
    }

    pub fn add_transaction(&self, transaction: NewTransaction) -> Result<Transaction> {
        let conn = self.conn.lock().unwrap();
        let date = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        
        let description = transaction.description.unwrap_or_else(|| "Untitled".to_string());
        let category = transaction.category.unwrap_or_else(|| "Other".to_string());
        
        conn.execute(
            "INSERT INTO transactions (amount, description, category, date) VALUES (?1, ?2, ?3, ?4)",
            [
                &transaction.amount.to_string(),
                &description,
                &category,
                &date,
            ],
        )?;

        let id = conn.last_insert_rowid();
        
        Ok(Transaction {
            id,
            amount: transaction.amount,
            description,
            category,
            date,
        })
    }

    pub fn get_transactions(&self, limit: Option<i64>) -> Result<Vec<Transaction>> {
        let conn = self.conn.lock().unwrap();
        let query = match limit {
            Some(l) => format!("SELECT id, amount, description, category, date FROM transactions ORDER BY date DESC LIMIT {}", l),
            None => "SELECT id, amount, description, category, date FROM transactions ORDER BY date DESC".to_string(),
        };

        let mut stmt = conn.prepare(&query)?;
        let transactions = stmt.query_map([], |row| {
            Ok(Transaction {
                id: row.get(0)?,
                amount: row.get(1)?,
                description: row.get(2)?,
                category: row.get(3)?,
                date: row.get(4)?,
            })
        })?;

        transactions.collect()
    }

    pub fn get_monthly_balance(&self) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let current_month = chrono::Local::now().format("%Y-%m").to_string();
        
        let balance: i64 = conn.query_row(
            "SELECT COALESCE(SUM(amount), 0) FROM transactions WHERE date LIKE ?1",
            [format!("{}%", current_month)],
            |row| row.get(0),
        )?;

        Ok(balance)
    }

    pub fn get_all_time_balance(&self) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        
        let balance: i64 = conn.query_row(
            "SELECT COALESCE(SUM(amount), 0) FROM transactions",
            [],
            |row| row.get(0),
        )?;

        Ok(balance)
    }

    pub fn export_transactions_csv(&self) -> Result<String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, amount, description, category, date FROM transactions ORDER BY date DESC"
        )?;
        
        let mut csv = String::from("ID,Amount,Description,Category,Date\n");
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
            ))
        })?;

        for row in rows {
            let (id, amount, desc, cat, date) = row?;
            let dollars = (amount as f64) / 100.0;
            csv.push_str(&format!("{},{:.2},{},{},{}\n", id, dollars, desc, cat, date));
        }

        Ok(csv)
    }

    pub fn delete_transaction(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM transactions WHERE id = ?1", [id])?;
        Ok(())
    }

    pub fn get_category_totals(&self) -> Result<Vec<(String, i64)>> {
        let conn = self.conn.lock().unwrap();
        let current_month = chrono::Local::now().format("%Y-%m").to_string();
        
        let mut stmt = conn.prepare(
            "SELECT category, SUM(amount) as total 
             FROM transactions 
             WHERE date LIKE ?1 AND amount < 0
             GROUP BY category 
             ORDER BY total ASC"
        )?;
        
        let results = stmt.query_map([format!("{}%", current_month)], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?;
        
        results.collect()
    }

    pub fn get_categories(&self) -> Result<Vec<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT name FROM categories ORDER BY is_default DESC, name ASC")?;
        
        let categories = stmt.query_map([], |row| row.get(0))?;
        categories.collect()
    }

    pub fn add_category(&self, name: String) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO categories (name, is_default) VALUES (?1, 0)",
            [name],
        )?;
        Ok(())
    }

    pub fn delete_category(&self, name: String) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM categories WHERE name = ?1 AND is_default = 0",
            [name],
        )?;
        Ok(())
    }

    pub fn get_available_months(&self) -> Result<Vec<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT DISTINCT strftime('%Y-%m', date) as month 
             FROM transactions 
             ORDER BY month DESC"
        )?;
        
        let months = stmt.query_map([], |row| row.get(0))?;
        months.collect()
    }

    pub fn get_balance_for_month(&self, month: String) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        
        let balance: i64 = conn.query_row(
            "SELECT COALESCE(SUM(amount), 0) FROM transactions WHERE date LIKE ?1",
            [format!("{}%", month)],
            |row| row.get(0),
        )?;

        Ok(balance)
    }

    pub fn get_transactions_for_month(&self, month: String, limit: Option<i64>) -> Result<Vec<Transaction>> {
        let conn = self.conn.lock().unwrap();
        let base_query = format!(
            "SELECT id, amount, description, category, date FROM transactions WHERE date LIKE '{}%' ORDER BY date DESC",
            month
        );
        
        let query = match limit {
            Some(l) => format!("{} LIMIT {}", base_query, l),
            None => base_query,
        };

        let mut stmt = conn.prepare(&query)?;
        let transactions = stmt.query_map([], |row| {
            Ok(Transaction {
                id: row.get(0)?,
                amount: row.get(1)?,
                description: row.get(2)?,
                category: row.get(3)?,
                date: row.get(4)?,
            })
        })?;

        transactions.collect()
    }

    pub fn get_category_totals_for_month(&self, month: String) -> Result<Vec<(String, i64)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT category, SUM(amount) as total 
             FROM transactions 
             WHERE amount < 0 AND date LIKE ?1
             GROUP BY category 
             ORDER BY total ASC"
        )?;

        let results = stmt.query_map([format!("{}%", month)], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?;
        
        results.collect()
    }
}
