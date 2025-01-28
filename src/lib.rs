#[cfg(test)]
mod tests {
    use practice_macro::{comp, cfunc};


    #[test]
    fn it_works() {
        let empty: Vec<i32> = comp![x for x in []].collect();
        assert_eq!(empty.len(), 0);

        let one: Vec<i32> = comp![x for x in [1]].collect();
        assert_eq!(one.len(), 1);

        let unrelated: Vec<&str> = comp!["hello!" for x in [1, 2, 3]].collect();
        assert_eq!(unrelated.len(), 3);
        assert_eq!(*unrelated.first().unwrap(), "hello!");

        let filtered: Vec<i32> = comp![x*x for x in [1, 2, 3, 4, 5, 6] if x % 2 == 0].collect();
        assert_eq!(filtered, vec![4, 16, 36]);
    }

    #[test]
    fn function_macros() {
        assert_eq!(newFunction(5), 6);
    }

    cfunc!{
        int newFunction(int input) {
            return input + 1;
        }
    }
}
