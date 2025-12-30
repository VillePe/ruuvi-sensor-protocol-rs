pub fn type_has_default_traits<T: Sized + Send + Sync + Unpin>() {}

macro_rules! floating_value_test {
    (
        $name: ident,
        $values: expr,
        $expected: expr,
        $epsilon: expr
    ) => {
         mod $name {
             use super::*;

             #[test]
             fn $name() {
                 let ok = ($values - $expected).abs() < $epsilon;
                 let equality_operator = if $values == $expected { "==" } else if ok { "~=" } else { "!=" };
                 println!("{0} {1} {2}", $values, equality_operator, $expected);
                 assert!(
                     ($values - $expected).abs() < $epsilon
                 )
             }
         }
    };
}

pub(crate) use floating_value_test;