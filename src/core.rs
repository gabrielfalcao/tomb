pub fn version() -> String {
    let version = env!("CARGO_PKG_VERSION");
    String::from(version)
}
pub fn author() -> String {
    let authors = env!("CARGO_PKG_AUTHORS").split(",");
    let authors: Vec<&str> = authors.collect();
    String::from(authors[0])
}
