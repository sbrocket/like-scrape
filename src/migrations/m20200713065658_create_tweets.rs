use diesel::{
    connection::SimpleConnection, migration::RunMigrationsError, sqlite::SqliteConnection,
};

// TODO: Most of this is boilerplate...create a macro for simple SQL migrations?
pub struct Migration<'a> {
    pub conn: &'a SqliteConnection,
}

impl<'a> diesel::migration::Migration for Migration<'a> {
    fn version(&self) -> &str {
        version_from_filename!()
    }

    fn run(&self, _: &dyn SimpleConnection) -> Result<(), RunMigrationsError> {
        // Use the full connection here, rather than the SimpleConnection argument, to verify that
        // passing the full connection through is working properly, even through this particular
        // migration is simple.
        // TODO: This is just a placeholder to test the migration wiring; define real table structure.
        self.conn.batch_execute(
            r#"CREATE TABLE "tweets" (
            "id" INTEGER NOT NULL PRIMARY KEY,
            "foo" TEXT
         );"#,
        )?;
        Ok(())
    }

    fn revert(&self, _: &dyn SimpleConnection) -> Result<(), RunMigrationsError> {
        self.conn.batch_execute(r#"DROP TABLE "tweets";"#)?;
        Ok(())
    }
}
