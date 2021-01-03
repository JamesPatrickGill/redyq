extern crate redis;
extern crate strum_macros;

pub mod events;
pub mod job;
pub mod queue;
pub mod worker;

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {}
}
