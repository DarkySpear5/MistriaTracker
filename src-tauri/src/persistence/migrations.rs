use rusqlite::Connection;

pub fn apply(connection: &Connection) -> rusqlite::Result<()> {
    apply_sql(
        connection,
        include_str!("../../migrations/0001_initial.sql"),
    )?;
    apply_sql(
        connection,
        include_str!("../../migrations/0002_imported_backups.sql"),
    )
}

pub(crate) fn apply_sql(connection: &Connection, sql: &str) -> rusqlite::Result<()> {
    connection.execute_batch(sql)
}
