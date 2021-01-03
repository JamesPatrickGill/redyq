use crate::events::QueueEvent;
use crate::job::Job;
use redis::{Client, Commands};

use std::str::FromStr;

use log::info;

#[derive(Clone, Copy)]
struct QueueData {
    queue_name: &'static str,
}

#[derive(Clone, Copy)]
pub struct Queue {
    redis_url: &'static str,
    queue_data: QueueData,
}

impl Queue {
    pub fn add(self, job_name: &str, message: serde_json::Value) {
        let job: Job = Job {
            name: job_name.to_owned(),
            message,
            retry_count: 0,
        };
        let mut con = Client::open(self.redis_url)
            .unwrap()
            .get_connection()
            .unwrap();
        let _: () = con
            .lpush(
                format!("{}{}", self.queue_data.queue_name, ".waiting"),
                serde_json::to_string(&job).unwrap(),
            )
            .unwrap();
        let _: () = con
            .publish(
                format!("{}.queue-events", self.queue_data.queue_name),
                "AddWaiting",
            )
            .unwrap();
    }
}

pub fn create_queue(
    queue_name: &'static str,
    redis_url: &'static str,
) -> Result<Queue, redis::RedisError> {
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

    event_subscriber(queue_name, redis_url);

    Ok(Queue {
        queue_data: QueueData { queue_name },
        redis_url,
    })
}

fn event_subscriber(queue_name: &str, redis_url: &str) {
    let client = redis::Client::open(redis_url).unwrap();
    let mut con = client.get_connection().unwrap();

    let owned_queue_name = queue_name.to_owned();

    std::thread::spawn(move || {
        let mut pubsub = con.as_pubsub();
        pubsub
            .subscribe(format!("{}.queue-events", owned_queue_name))
            .unwrap();

        loop {
            let msg: String = pubsub.get_message().unwrap().get_payload().unwrap();
            let msg_as_queue_event: QueueEvent = QueueEvent::from_str(msg.as_str()).unwrap();

            match msg_as_queue_event {
                QueueEvent::AddWaiting => {
                    println!("hmm");
                }
            };
        }
    });
}
