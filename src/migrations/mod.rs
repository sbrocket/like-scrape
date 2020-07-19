macro_rules! version_from_filename {
    () => {{
        // The module/file name format is validated in build.rs
        let file = std::path::Path::new(file!());
        let file = file.file_name().and_then(std::ffi::OsStr::to_str).unwrap();
        file[1..].split('_').next().unwrap()
    }};
}

// Migration modules need to be manually declared here. The build script apparently can't generate
// these lines because then it looks for the modules in OUT_DIR.
mod m20200713065658_create_tweets;

// build.rs automatically generates the contents of this file as migrations are added or removed.
//
// The generated contents declare a function with the following prototype which returns an collection
// of all embedded migrations:
//
//   pub fn embedded_migrations<'a>(conn: &'a SqliteConnection) -> Vec<Box<dyn diesel::migration::Migration + 'a>>;
//
// Passing the connection here is a hack to get the migration implementations the full
// Connection instead of SimpleConnection. (https://github.com/diesel-rs/diesel/issues/2457)
include!(concat!(env!("OUT_DIR"), "/migrations_mod.rs"));
