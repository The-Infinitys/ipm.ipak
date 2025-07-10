



#[macro_export] 
macro_rules! dprintln {
    
    
    ($($arg:tt)*) => ({
        #[cfg(debug_assertions)] 
        println!($($arg)*);
    });
}













#[macro_export] 
macro_rules! dprint {
    
    ($($arg:tt)*) => ({
        #[cfg(debug_assertions)] 
        print!($($arg)*);
    });
}
