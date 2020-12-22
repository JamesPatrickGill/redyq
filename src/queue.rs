use crate::job::Job;
use redis::Commands;

struct QueueData {
    queue_name: String,
}

pub struct Queue {
    redis_client: redis::Client,
    queue_data: QueueData,
    fail_watcher_handle: std::thread::JoinHandle<()>,
    complete_watcher_handle: std::thread::JoinHandle<()>,
}

impl Queue {
    pub fn add(self, job_name: String, message: serde_json::Value) {
        let job: Job = Job {
            name: job_name,
            message,
            retry_count: 0,
        };
        let mut con = self.redis_client.get_connection().unwrap();
        let _: () = con
            .lpush(
                format!("{}{}", self.queue_data.queue_name, ".waiting"),
                serde_json::to_string(&job).unwrap(),
            )
            .unwrap();
    }
}

pub fn create_queue(queue_name: String, redis_url: String) -> Result<Queue, redis::RedisError> {
    let client = redis::Client::open(redis_url)?;
    let fail_watcher_handle = start_fail_watcher(queue_name.to_string());
    let complete_watcher_handle = start_complete_watcher(queue_name.to_string());
    Ok(Queue {
        fail_watcher_handle,
        complete_watcher_handle,
        queue_data: QueueData { queue_name },
        redis_client: client,
    })
}

fn start_fail_watcher(queue_name: String) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let client = redis::Client::open("redis://127.0.0.1:6379/").unwrap();
        let mut con = client.get_connection().unwrap();
        loop {
            let (_, job_string): (String, String) = con
                .brpop(format!("{}{}", &queue_name, ".failed"), 0)
                .unwrap();
            let failed_job: Job = serde_json::from_str(&job_string).unwrap();
            if failed_job.retry_count <= 5 {
                let retried_job = Job {
                    retry_count: failed_job.retry_count + 1,
                    ..failed_job
                };
                let _: () = con
                    .lpush(
                        format!("{}{}", &queue_name, ".waiting"),
                        serde_json::to_string(&retried_job).unwrap(),
                    )
                    .unwrap();
            }
        }
    })
}

fn start_complete_watcher(queue_name: String) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let client = redis::Client::open("redis://127.0.0.1:6379/").unwrap();
        let mut con = client.get_connection().unwrap();
        loop {
            let (_, _): (String, String) = con
                .brpop(format!("{}{}", &queue_name, ".completed"), 0)
                .unwrap();
        }
    })
}
