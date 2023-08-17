/// parse ids from comma-separated numbers
pub fn get_ids_from_str(s: &str) -> Vec<i32> {
    s.split(',').filter_map(|x| x.parse().ok()).collect()
}
