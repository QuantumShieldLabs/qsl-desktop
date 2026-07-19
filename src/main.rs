fn main() {
    println!(
        "qsl-desktop {} (bootstrap placeholder; no application functionality)",
        env!("CARGO_PKG_VERSION")
    );
}

#[cfg(test)]
mod tests {
    #[test]
    fn version_is_nonempty() {
        assert!(!env!("CARGO_PKG_VERSION").is_empty());
    }
}
