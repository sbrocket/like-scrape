use {
    anyhow::Result,
    assert_matches::*,
    std::{env, fs, io::Write, path::Path},
};

const MIGRATIONS_DIR: &str = "src/migrations/";

fn main() -> Result<()> {
    // Locate all migrations in src/migrations/
    let mut migrations = vec![];
    for entry in fs::read_dir(MIGRATIONS_DIR)? {
        let path = entry?.path();
        assert!(path.is_file(), "{} should not contain subdirectories", MIGRATIONS_DIR);
        assert_matches!(
            path.extension(),
            Some(ext) if ext == "rs",
            "{} should only contain Rust source",
            MIGRATIONS_DIR
        );
        let file_name = path.file_name().unwrap().to_str().unwrap();
        if file_name != "mod.rs" {
            migrations.push(file_name.to_string());
        }
    }

    // Validate that migration module names start with ISO-8601 basic format datetime
    let bad_names: Vec<_> = migrations.iter().filter(|n| !validate_migration_name(n)).collect();
    assert!(
        bad_names.is_empty(),
        "Bad migration mod names, should be named 'm[ISO-8601 datetime]_[name].rs': {:?}",
        bad_names
    );

    // Generate source included in src/migrations/mod.rs
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let out_path = Path::new(&out_dir).join("migrations_mod.rs");
    let mut out_file = fs::File::create(out_path)?;
    write_migrations_mod(&mut out_file, &migrations);

    // Rerun this build script if the migrations directory changes. rerun-if-changed checks the
    // mtime. directory mtime should change whenever a module is added, removed, or its name is
    // changed, so we shouldn't need to track the individual migration files.
    println!("cargo:rerun-if-changed={}", MIGRATIONS_DIR);

    Ok(())
}

fn validate_migration_name(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some('m') => {}
        _ => return false,
    }

    let rest = chars.as_str();
    if let Some(idx) = rest.find('_') {
        let datetime = &rest[..idx];
        return datetime.len() == 14 && datetime.chars().all(|c| char::is_ascii_digit(&c));
    }
    false
}

fn write_migrations_mod<W: Write, S: AsRef<str>>(out: &mut W, migrations: &[S]) {
    let create_migrations = migrations
        .iter()
        .map(|n| n.as_ref())
        .map(|n| &n[..n.len() - 3])
        .map(|n| format!("Box::new({}::Migration{{conn}}),", n))
        .collect::<Vec<_>>()
        .join("\n        ");

    write!(
        out,
        "\
use {{
    diesel::sqlite::SqliteConnection,
    diesel::migration::Migration,
}};

// TODO: Passing the connection here is a hack to get the migration implementations the full
// connection instead of SimpleConnection.
pub fn embedded_migrations<'a>(conn: &'a SqliteConnection) -> Vec<Box<dyn Migration + 'a>> {{
    vec![
        {}
    ]
}}
\
    ",
        create_migrations
    )
    .unwrap();
}
