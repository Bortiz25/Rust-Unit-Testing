use std::{ cmp::Ordering, collections::HashMap, hash::Hash };
use std::sync::atomic::{ AtomicI32 };
use std::sync::{ Arc, Mutex };
use std::thread;

struct File {
    content: String,
    count_map: HashMap<String, u32>,
    ordered_content: Vec<String>,
}

impl File {
    fn new(content: &str) -> Self {
        Self {
            content: content.to_string(),
            count_map: HashMap::new(),
            ordered_content: Vec::new(),
        }
    }

    fn parse_and_count(&mut self, substr: &str) -> u32 {
        let count = self.content.matches(substr).count() as u32;
        *self.count_map.entry(substr.to_string()).or_insert(0) += count;
        count
    }

    fn order_content_alphabetically(&mut self) -> Vec<String> {
        let mut sorted_content: Vec<_> = self.count_map.iter().collect();
        sorted_content.sort_by(|a, b| a.0.cmp(&b.0));
        let ordered = sorted_content
            .iter()
            .map(|(substr, _)| substr.to_string())
            .collect::<Vec<_>>();
        self.ordered_content = ordered.clone();
        ordered
    }

    fn print_counts(&self) {
        for (substr, count) in &self.count_map {
            println!("Substring: {}, Count: {}", substr, count);
        }
    }

    fn get_count(&self, substr: &str) -> Option<&u32> {
        self.count_map.get(substr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_parse_and_count() {
        let mut file = File::new("hello world hello there hello");
        let hello_count = file.parse_and_count("hello");
        assert_eq!(hello_count, 3);
    }

    #[test]
    fn test_get_count() {
        let mut file = File::new("hello world hello there hello");
        let _hello_count = file.parse_and_count("world");
        let world_count = file.get_count("world");
        assert_eq!(world_count, Some(&1));
    }

    #[test]
    fn test_concurrent_parse_and_count() {
        let file_arc = Arc::new(Mutex::new(File::new("hello world hello there hello")));
        let threads: Vec<_> = (0..5)
            .map(|_| {
                let file_arc = Arc::clone(&file_arc);
                thread::spawn(move || {
                    let mut file = file_arc.lock().unwrap();
                    let _hello_count = file.parse_and_count("hello");
                })
            })
            .collect();

        for thread in threads {
            thread.join().unwrap();
        }

        let file = file_arc.lock().unwrap();
        let hello_count = file.get_count("hello");
        assert_eq!(hello_count, Some(&15));
    }

    // beginning of tests utilizing proptest
    proptest! {
        #[test]
        fn test_parse_and_count_property(content in ".*", substr in ".*") {
            let mut file = File::new(&content);
            let count = file.parse_and_count(&substr);
            let count_match = file.content.matches(&substr).count() as u32;
            prop_assert_eq!(count, count_match);
        }

        #[test]
        fn test_get_count_property(content in ".*", substr in ".*") {
            let mut file = File::new(&content);
            let count = file.parse_and_count(&substr);
            let retrieved_count = file.get_count(&substr);
            prop_assert_eq!(Some(&count), retrieved_count);
        }
    }
}

fn main() {
    let mut file = File::new("hello world hello there hello");
    file.parse_and_count("o");
    file.parse_and_count("world");
    file.parse_and_count("there");
    file.parse_and_count("rust");

    file.print_counts();
    file.order_content_alphabetically();

    println!("Ordered Content: {:?}", file.ordered_content);
}
