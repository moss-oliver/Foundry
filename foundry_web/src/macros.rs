
//See: https://stackoverflow.com/questions/46136169/overcoming-local-ambiguity-multiple-parsing-options-in-rust-macros
/*#[macro_export]
macro_rules! tt_match {
    ($left:tt in $( ($($right:tt)|*) => $result:tt );+ $(; _ => $default:tt)* $(;)* ) => {{
        macro_rules! __internal {
            $( $( ($right) => $result; )* )+
            $( ($left) => $default )*
        }
        __internal!{ $left }
    }}
}*/

/*
#[macro_export]
macro_rules! html2 {
    ([$($left:tt)*] [$ex_close:tt $($stack:tt)*] </$close:ident> $($tail:tt)*) => {
        tt_match!{
            $close in
                ($ex_close) => {
                    html2!{ [$($left)* "</", stringify!($close), ">",] [$($stack)*] $($tail)* }
                };
                _ => { compile_error!(concat!("Expected \"", stringify!($ex_close), "\"; found \"", stringify!($close), "\"")) }
        }
    };
    ([$($left:tt)*] [$($stack:tt)*] <$open:ident $($tail:tt)*) => {
        html2!{ [$($left)* "<", stringify!($open),] [$open $($stack)*] $($tail)* }
    };
    ([$($left:tt)*] [$current:tt $($stack:tt)*] > $($tail:tt)*) => {
        tt_match!{ $current in
            // auto closing
            (img | br) => {
                html2!{ [$($left)* ">",] [$($stack)*] $($tail)* }
            };
            // not auto closing
            _ => {
                html2!{ [$($left)* ">",] [$current $($stack)*] $($tail)* }
            }
        }
    };
    ([$($left:tt)*] $stack:tt $key:tt=$value:tt $($tail:tt)*) => {
        html2!{ [$($left)* " ", stringify!($key), "=", stringify!($value),] $stack $($tail)* }
    };
    ([$($left:tt)*] $stack:tt $text:tt $($tail:tt)*) => {
        html2!{ [$($left)* $text,] $stack $($tail)* }
    };
    ([$($left:tt)*] []) => {
        concat!($($left)*)
    };
}
*/

/*
#[macro_export]
macro_rules! html2 {
    ([$($left:tt)*] [$ex_close:tt $($stack:tt)*] </$close:ident> $($tail:tt)*) => {
        tt_match!{
            $close in
                ($ex_close) => {
                      html2!{ [$($left)* "</", stringify!($close), ">",] [$($stack)*] $($tail)* }
                };
                _ => { compile_error!(concat!("Expected \"", stringify!($ex_close), "\"; found \"", stringify!($close), "\"")) }
        }
    };
    ([$($left:tt)*] [$($stack:tt)*] <$open:ident $($tail:tt)*) => {
        html2!{ [$($left)* "<", stringify!($open),] [$open $($stack)*] $($tail)* }
    };
    ([$($left:tt)*] [$current:tt $($stack:tt)*] > $($tail:tt)*) => {
        tt_match!{ $current in
            // auto closing
            (img | br) => {
                Box::new(HtmlNode::new(stringify!($current), vec!(), vec!(
                    html2!{ [$($left)* ">",] [$($stack)*] $($tail)* }
                )))
            };
            // not auto closing
            _ => {
                Box::new(HtmlNode::new(stringify!($current), vec!(), vec!(
                    html2!{ [$($left)* ">",] [$current $($stack)*] $($tail)* }
                )))
            }
        }
    };
    ([$($left:tt)*] $stack:tt $key:tt=$value:tt $($tail:tt)*) => {
        html2!{ [$($left)* " ", stringify!($key), "=", stringify!($value),] $stack $($tail)* }
    };
    ([$($left:tt)*] $stack:tt $text:tt $($tail:tt)*) => {
        html2!{ [$($left)* $text,] $stack $($tail)* }
    };
    ([$($left:tt)*] []) => {
        $($left)*
    };
}
*/

/*#[macro_export]
macro_rules! html {
    ($($left:tt)*) => {
        html2!([] [] $($left)*)
    }
}*/


/*#[macro_export]
macro_rules! html {
    ($($t:tt)*) => {{
        struct _X;
        impl _X {
            html_impl!($($t)*);
        }
        _X::output()
    }}
}
*/
