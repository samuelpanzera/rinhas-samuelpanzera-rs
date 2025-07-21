use r2d2_sqlite::SqliteConnectionManager;
use r2d2::Pool;

#[derive(Debug)]
struct ConnectionCustomizer;

impl r2d2::CustomizeConnection<rusqlite::Connection, rusqlite::Error> for ConnectionCustomizer {
    fn on_acquire(&self, conn: &mut rusqlite::Connection) -> Result<(), rusqlite::Error> {
        conn.execute_batch(
            "
            PRAGMA journal_mode = WAL;  
            PRAGMA synchronous = NORMAL;
            PRAGMA cache_size = -32768;
            PRAGMA temp_store = MEMORY;
            PRAGMA mmap_size = 134217728;
            "
        )?;
        Ok(())
    }
}


pub fn create_optimized_pool(db_path: &str) -> Pool<SqliteConnectionManager> {
    let manager = SqliteConnectionManager::file(db_path);
   
    Pool::builder()
        .max_size(8) 
        .connection_customizer(Box::new(ConnectionCustomizer)) 
        .build(manager)
        .expect("Falha ao criar o pool de conexões otimizado.")
}
