pub fn searchInVector(vector: &Vec<String>, search_query: &String) -> Vec<String> {
    return vector
        .iter()
        .filter(|skill| skill.contains(search_query))
        .cloned()
        .collect();
}
