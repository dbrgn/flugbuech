pub fn is_valid_tracktype(value: &str) -> bool {
    ["free_flight", "flat_triangle", "fai_triangle"].contains(&value)
}
