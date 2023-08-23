/// parse ids from comma-separated numbers
pub fn get_ids_from_str(s: &str) -> Vec<i32> {
    s.split(',').filter_map(|x| x.parse().ok()).collect()
}

pub fn remove_enter(s: &str) -> String {
    s.replace('\n', "")
}

pub fn trim_end_inplace(s: &mut String) {
    s.truncate(s.trim_end().len());
}
