macro_rules! path_join {
    ($base:expr, $($x:expr),*) => {
        {
            let mut temp = $base;
            $(
                temp.push($x);
            )*
            temp
        }
    }
}
