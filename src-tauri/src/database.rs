use rusqlite::{Connection, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;
use csv::ReaderBuilder;

#[derive(Debug, Serialize, Deserialize)]
pub struct Container {
    pub id: i64,
    pub name: String,
    pub created_at: String,
    pub is_default: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Transaction {
    pub id: i64,
    pub amount: i64,
    pub description: String,
    pub category: String,
    pub date: String,
    pub container_id: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewTransaction {
    pub amount: i64,
    pub description: Option<String>,
    pub category: Option<String>,
    pub container_id: i64,
}

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn new(db_path: PathBuf) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS containers (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                created_at TEXT NOT NULL,
                is_default INTEGER NOT NULL DEFAULT 0
            )",
            [],
        )?;

        let container_count: i64 = conn.query_row("SELECT COUNT(*) FROM containers", [], |row| row.get(0))?;
        if container_count == 0 {
            let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
            conn.execute(
                "INSERT INTO containers (name, created_at, is_default) VALUES (?1, ?2, 1)",
                ["Personal", &now],
            )?;
        }

        conn.execute(
            "CREATE TABLE IF NOT EXISTS transactions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                amount INTEGER NOT NULL,
                description TEXT NOT NULL,
                category TEXT NOT NULL,
                date TEXT NOT NULL,
                container_id INTEGER NOT NULL DEFAULT 1,
                FOREIGN KEY (container_id) REFERENCES containers(id) ON DELETE CASCADE
            )",
            [],
        )?;

        let has_container_id: Result<i64, _> = conn.query_row(
            "SELECT COUNT(*) FROM pragma_table_info('transactions') WHERE name='container_id'",
            [],
            |row| row.get(0)
        );
        
        if let Ok(0) = has_container_id {
            conn.execute(
                "ALTER TABLE transactions ADD COLUMN container_id INTEGER NOT NULL DEFAULT 1",
                [],
            )?;
        }

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
            "INSERT INTO transactions (amount, description, category, date, container_id) VALUES (?1, ?2, ?3, ?4, ?5)",
            [
                &transaction.amount.to_string(),
                &description,
                &category,
                &date,
                &transaction.container_id.to_string(),
            ],
        )?;

        let id = conn.last_insert_rowid();
        
        Ok(Transaction {
            id,
            amount: transaction.amount,
            description,
            category,
            date,
            container_id: transaction.container_id,
        })
    }

    pub fn get_transactions(&self, container_id: i64, limit: Option<i64>) -> Result<Vec<Transaction>> {
        let conn = self.conn.lock().unwrap();
        let query = match limit {
            Some(l) => format!("SELECT id, amount, description, category, date, container_id FROM transactions WHERE container_id = {} ORDER BY date DESC LIMIT {}", container_id, l),
            None => format!("SELECT id, amount, description, category, date, container_id FROM transactions WHERE container_id = {} ORDER BY date DESC", container_id),
        };

        let mut stmt = conn.prepare(&query)?;
        let transactions = stmt.query_map([], |row| {
            Ok(Transaction {
                id: row.get(0)?,
                amount: row.get(1)?,
                description: row.get(2)?,
                category: row.get(3)?,
                date: row.get(4)?,
                container_id: row.get(5)?,
            })
        })?;

        transactions.collect()
    }

    pub fn update_transaction(
        &self,
        id: i64,
        amount: i64,
        description: String,
        category: String,
    ) -> Result<Transaction> {
        let conn = self.conn.lock().unwrap();
        
        conn.execute(
            "UPDATE transactions SET amount = ?1, description = ?2, category = ?3 WHERE id = ?4",
            [&amount.to_string(), &description, &category, &id.to_string()],
        )?;

        let transaction = conn.query_row(
            "SELECT id, amount, description, category, date, container_id FROM transactions WHERE id = ?1",
            [id],
            |row| {
                Ok(Transaction {
                    id: row.get(0)?,
                    amount: row.get(1)?,
                    description: row.get(2)?,
                    category: row.get(3)?,
                    date: row.get(4)?,
                    container_id: row.get(5)?,
                })
            },
        )?;

        Ok(transaction)
    }

    pub fn get_monthly_balance(&self, container_id: i64) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let current_month = chrono::Local::now().format("%Y-%m").to_string();
        
        let balance: i64 = conn.query_row(
            "SELECT COALESCE(SUM(amount), 0) FROM transactions WHERE container_id = ?1 AND date LIKE ?2",
            [&container_id.to_string(), &format!("{}%", current_month)],
            |row| row.get(0),
        )?;

        Ok(balance)
    }

    pub fn get_all_time_balance(&self, container_id: i64) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        
        let balance: i64 = conn.query_row(
            "SELECT COALESCE(SUM(amount), 0) FROM transactions WHERE container_id = ?1",
            [container_id],
            |row| row.get(0),
        )?;

        Ok(balance)
    }

    pub fn export_transactions_csv(&self, container_id: i64) -> Result<String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, amount, description, category, date FROM transactions WHERE container_id = ?1 ORDER BY date DESC"
        )?;
        
        let mut csv = String::from("ID,Amount,Description,Category,Date\n");
        let rows = stmt.query_map([container_id], |row| {
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

    pub fn get_category_totals(&self, container_id: i64) -> Result<Vec<(String, i64)>> {
        let conn = self.conn.lock().unwrap();
        let current_month = chrono::Local::now().format("%Y-%m").to_string();
        
        let mut stmt = conn.prepare(
            "SELECT category, SUM(amount) as total 
             FROM transactions 
             WHERE container_id = ?1 AND date LIKE ?2 AND amount < 0
             GROUP BY category 
             ORDER BY total ASC"
        )?;
        
        let results = stmt.query_map([&container_id.to_string(), &format!("{}%", current_month)], |row| {
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

    pub fn get_available_months(&self, container_id: i64) -> Result<Vec<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT DISTINCT strftime('%Y-%m', date) as month 
             FROM transactions 
             WHERE container_id = ?1
             ORDER BY month DESC"
        )?;
        
        let months = stmt.query_map([container_id], |row| row.get(0))?;
        months.collect()
    }

    pub fn get_balance_for_month(&self, container_id: i64, month: String) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        
        let balance: i64 = conn.query_row(
            "SELECT COALESCE(SUM(amount), 0) FROM transactions WHERE container_id = ?1 AND date LIKE ?2",
            [&container_id.to_string(), &format!("{}%", month)],
            |row| row.get(0),
        )?;

        Ok(balance)
    }

    pub fn get_transactions_for_month(&self, container_id: i64, month: String, limit: Option<i64>) -> Result<Vec<Transaction>> {
        let conn = self.conn.lock().unwrap();
        let base_query = format!(
            "SELECT id, amount, description, category, date, container_id FROM transactions WHERE container_id = {} AND date LIKE '{}%' ORDER BY date DESC",
            container_id, month
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
                container_id: row.get(5)?,
            })
        })?;

        transactions.collect()
    }

    pub fn get_category_totals_for_month(&self, container_id: i64, month: String) -> Result<Vec<(String, i64)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT category, SUM(amount) as total 
             FROM transactions 
             WHERE container_id = ?1 AND amount < 0 AND date LIKE ?2
             GROUP BY category 
             ORDER BY total ASC"
        )?;

        let results = stmt.query_map([&container_id.to_string(), &format!("{}%", month)], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?;
        
        results.collect()
    }

    pub fn get_containers(&self) -> Result<Vec<Container>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, name, created_at, is_default FROM containers ORDER BY is_default DESC, created_at ASC")?;
        
        let containers = stmt.query_map([], |row| {
            Ok(Container {
                id: row.get(0)?,
                name: row.get(1)?,
                created_at: row.get(2)?,
                is_default: row.get::<_, i64>(3)? == 1,
            })
        })?;
        
        containers.collect()
    }

    pub fn add_container(&self, name: String) -> Result<Container> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        
        conn.execute(
            "INSERT INTO containers (name, created_at, is_default) VALUES (?1, ?2, 0)",
            [&name, &now],
        )?;

        let id = conn.last_insert_rowid();
        
        Ok(Container {
            id,
            name,
            created_at: now,
            is_default: false,
        })
    }

    pub fn delete_container(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        
        let is_default: i64 = conn.query_row(
            "SELECT is_default FROM containers WHERE id = ?1",
            [id],
            |row| row.get(0),
        )?;
        
        if is_default == 1 {
            return Err(rusqlite::Error::InvalidParameterName("Cannot delete default container".to_string()));
        }
        
        conn.execute("DELETE FROM containers WHERE id = ?1", [id])?;
        Ok(())
    }

    pub fn update_container(&self, id: i64, name: String) -> Result<Container> {
        let conn = self.conn.lock().unwrap();
        
        conn.execute(
            "UPDATE containers SET name = ?1 WHERE id = ?2",
            [&name, &id.to_string()],
        )?;

        let container = conn.query_row(
            "SELECT id, name, created_at, is_default FROM containers WHERE id = ?1",
            [id],
            |row| {
                Ok(Container {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    created_at: row.get(2)?,
                    is_default: row.get::<_, i64>(3)? == 1,
                })
            },
        )?;

        Ok(container)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImportResult {
    pub success_count: usize,
    pub error_count: usize,
    pub errors: Vec<String>,
}

impl Database {
    pub fn import_transactions_from_csv(
        &self,
        csv_content: String,
        container_id: i64,
        amount_column: usize,
        description_column: usize,
        category_column: usize,
        date_column: usize,
        skip_header: bool,
    ) -> Result<ImportResult> {
        let mut reader = ReaderBuilder::new()
            .has_headers(skip_header)
            .from_reader(csv_content.as_bytes());

        let mut success_count = 0;
        let mut error_count = 0;
        let mut errors = Vec::new();

        for (index, result) in reader.records().enumerate() {
            let row_num = if skip_header { index + 2 } else { index + 1 };
            
            match result {
                Ok(record) => {
                    let amount_str = record.get(amount_column).unwrap_or("").trim();
                    let description = record.get(description_column).unwrap_or("Imported").trim().to_string();
                    let category = record.get(category_column).unwrap_or("Other").trim().to_string();
                    let date_str = record.get(date_column).unwrap_or("").trim();

                    let amount_cents = match Self::parse_amount(amount_str) {
                        Ok(amt) => amt,
                        Err(e) => {
                            errors.push(format!("Row {}: Invalid amount '{}' - {}", row_num, amount_str, e));
                            error_count += 1;
                            continue;
                        }
                    };

                    let parsed_date = match Self::parse_date(date_str) {
                        Ok(date) => date,
                        Err(e) => {
                            errors.push(format!("Row {}: Invalid date '{}' - {}", row_num, date_str, e));
                            error_count += 1;
                            continue;
                        }
                    };

                    match self.insert_imported_transaction(
                        container_id,
                        amount_cents,
                        description,
                        category,
                        parsed_date,
                    ) {
                        Ok(_) => success_count += 1,
                        Err(e) => {
                            errors.push(format!("Row {}: Failed to insert - {}", row_num, e));
                            error_count += 1;
                        }
                    }
                }
                Err(e) => {
                    errors.push(format!("Row {}: Failed to parse CSV - {}", row_num, e));
                    error_count += 1;
                }
            }
        }

        Ok(ImportResult {
            success_count,
            error_count,
            errors,
        })
    }

    fn parse_amount(amount_str: &str) -> Result<i64, String> {
        let cleaned = amount_str
            .replace("$", "")
            .replace("€", "")
            .replace("£", "")
            .replace(",", "")
            .trim()
            .to_string();

        match cleaned.parse::<f64>() {
            Ok(amount) => Ok((amount * 100.0).round() as i64),
            Err(_) => Err(format!("Cannot parse as number")),
        }
    }

    fn parse_date(date_str: &str) -> Result<String, String> {
        let formats = vec![
            "%Y-%m-%d",
            "%m/%d/%Y",
            "%d/%m/%Y",
            "%Y/%m/%d",
            "%m-%d-%Y",
            "%d-%m-%Y",
            "%Y-%m-%d %H:%M:%S",
            "%m/%d/%Y %H:%M",
        ];

        for format in formats {
            if let Ok(parsed) = chrono::NaiveDateTime::parse_from_str(&format!("{} 00:00:00", date_str), "%Y-%m-%d %H:%M:%S") {
                return Ok(parsed.format("%Y-%m-%d %H:%M:%S").to_string());
            }
            if let Ok(parsed) = chrono::NaiveDateTime::parse_from_str(date_str, format) {
                return Ok(parsed.format("%Y-%m-%d %H:%M:%S").to_string());
            }
            if let Ok(parsed) = chrono::NaiveDate::parse_from_str(date_str, format) {
                let datetime = parsed.and_hms_opt(0, 0, 0).unwrap();
                return Ok(datetime.format("%Y-%m-%d %H:%M:%S").to_string());
            }
        }

        Err("Unsupported date format".to_string())
    }

    fn insert_imported_transaction(
        &self,
        container_id: i64,
        amount: i64,
        description: String,
        category: String,
        date: String,
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        
        conn.execute(
            "INSERT INTO transactions (amount, description, category, date, container_id) VALUES (?1, ?2, ?3, ?4, ?5)",
            [
                &amount.to_string(),
                &description,
                &category,
                &date,
                &container_id.to_string(),
            ],
        )?;

        Ok(())
    }
}
