use crate::job::Job;
use redis::Commands;
use serde_json::from_str;
use std::error::Error;

pub struct Worker {
    redis_client: redis::Client,
    queue_name: String,
}

impl Worker {
    pub fn start(self, processor: fn(Job) -> Result<(), Box<dyn Error>>) {
        std::thread::spawn(move || -> Option<()> {
            let mut con = self.redis_client.get_connection().ok()?;
            loop {
                let (_, job_string): (String, String) = con
                    .brpop(format!("{}{}", &self.queue_name[..], ".waiting"), 0)
                    .ok()?;
                match processor(from_str(&job_string).ok()?) {
                    Ok(()) => {
                        let _: () = con
                            .lpush(
                                format!("{}{}", &self.queue_name[..], ".completed"),
                                job_string,
                            )
                            .unwrap();
                    }
                    Err(_) => {
                        let _: () = con
                            .lpush(format!("{}{}", &self.queue_name[..], ".failed"), job_string)
                            .unwrap();
                    }
                }
            }
        });
    }
}

pub fn create_worker(queue_name: String) -> Result<Worker, redis::RedisError> {
    let client = redis::Client::open("redis://127.0.0.1:6379/")?;
    Ok(Worker {
        redis_client: client,
        queue_name,
    })
}
