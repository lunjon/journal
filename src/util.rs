pub fn get_date() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}
