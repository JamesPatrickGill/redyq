use crate::job::Job;
use redis::Commands;

use log::info;

struct QueueData {
    queue_name: String,
}

pub struct Queue {
    redis_client: redis::Client,
    queue_data: QueueData,
    event_subscription_handle: std::thread::JoinHandle<Result<(), redis::RedisError>>,
}

impl Queue {
    pub fn add(self, job_name: &str, message: serde_json::Value) {
        let job: Job = Job {
            name: job_name.to_owned(),
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
        let _: () = con
            .publish(
                format!("{}.queue-events", self.queue_data.queue_name),
                "job-added",
            )
            .unwrap();
    }
}

pub fn create_queue(queue_name: &str, redis_url: &str) -> Result<Queue, redis::RedisError> {
    let client = redis::Client::open(redis_url)?;

    let queue_already_exists: bool = client.get_connection()?.sismember("queues", queue_name)?;

    if queue_already_exists {
        info!("Existing queue with name \"{}\" found.", queue_name);
    } else {
        let queue_creation_successful: bool =
            client.get_connection()?.sadd("queues", queue_name)?;
        if queue_creation_successful {
            info!("Queue with name \"{}\" created", queue_name);
        }
    }

    let event_subscription_handle = event_subscriber(queue_name, redis_url)?;

    Ok(Queue {
        event_subscription_handle,
        queue_data: QueueData {
            queue_name: queue_name.to_owned(),
        },
        redis_client: client,
    })
}

fn event_subscriber(
    queue_name: &str,
    redis_url: &str,
) -> Result<std::thread::JoinHandle<Result<(), redis::RedisError>>, redis::RedisError> {
    let client = redis::Client::open(redis_url)?;
    let mut con = client.get_connection()?;

    let owned_queue_name = queue_name.to_owned();

    Ok(std::thread::spawn(move || {
        let mut pubsub = con.as_pubsub();
        pubsub.subscribe(format!("{}.queue-events", owned_queue_name))?;

        loop {
            let msg: String = pubsub.get_message()?.get_payload()?;
            println!("{}", msg)
        }
    }))
}
