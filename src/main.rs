#[cfg(test)]
mod tests {
    #[test]
    fn unit_test_example() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn test_simple_add() {
        use crate::simple_add;
        assert_eq!(5, simple_add(2, 3));
    }
}

pub fn simple_add(value1: i32, value2 :i32) -> i32 {
    value1 + value2
}

fn main() {
    println!("Hello from laputa!");
}
