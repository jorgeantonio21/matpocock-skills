use config_cli::{label, load};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args().nth(1).ok_or("usage: config_cli PATH")?;
    let capacity = load(&std::fs::read(path)?)?;
    println!("{}", label("capacity", &format!("{capacity:?}")));
    Ok(())
}
