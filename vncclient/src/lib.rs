pub mod vnc;

mod argparse;
mod error;
mod parsing;
// mod reporting;
mod util;

pub enum ThreadStatus {                                                            
    Complete,                                                                      
}  

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
