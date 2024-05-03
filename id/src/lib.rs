use rand::{distributions::Alphanumeric, Rng};

pub fn new(prefix: &str, length: usize) -> String {
    format!(
        "{}_{}",
        prefix,
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(length)
            .map(char::from)
            .collect::<String>()
    )
}
