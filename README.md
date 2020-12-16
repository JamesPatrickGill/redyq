# Redyq
<p align="center"><i>Though this project is small any contribution is welcome, be it in the form of ideas or code. So if you do wish to contribute feel free to use any of the github features at your disposal - issues, prs, etc</i></p>


## What it is 
Redyq is a simple redis queue interface which aims to allow painless distributed queue integration for any Rust service. It also strives to be fast and reliable as this software should not become the bottleneck of an application and as with any queue, data should never be lost. 

## Getting started
Though in infancy, redyq has a feature set that does provide the tools to both enqueue, dequeue and process jobs - plus some limited retrying capabilities. Keeping with the theme of simplicity, redyq has two key exposed functions `create_queue` and `create_worker`.

### `create_queue`

This function returns a `Queue` object that is our key object for adding objects to the queue using the `add` function. It currently jsut connect to a local redis instance on port 6379 but this will be configurable. 

At the moment the only argument to `create_queue` is the `queue_name` arg which is as you'd expect, the queue name. The function on this struct, `add`, takes two arguments, the job name (`job_name`) and the job message data (`message`) - note `message` takes a `serde_json::Value`. 

#### Example:

```rust
// This creates a queue
let queue: Queue = redyq::create_queue(String::from("example-queue").unwrap());

// This adds a job to the queue
queue.add(String::from("example-job", serde_json::from_str("{\"test_key\":\"test_value\"}").unwrap());

```

### `create_worker`

This function returns a `Worker` object that is our key object for consuming jobs for a queue. It currently jsut connect to a local redis instance on port 6379 but this will be configurable. 

At the moment the only argument to `create_worker` is the `queue_name` arg which is the queue name that you would like to be consumed. `Worker` has a single method on it which is the `start` method which take a processor which runs against every job from the queue and returns `Result<(), Box<dyn Error>` - if this result is Ok then the worker moves the job to a completed queue otherwise it moves it to a failed queue so that it can be retried. 

#### Example:

```rust
// This creates a Worker
let worker: Worker = redyq::create_worker(String::from("example-queue")).unwrap();

// This starts the Worker with a provessor which just prints the job name
worker.start(|job: Job| {
    println!("{}", job.name);
    Ok(())
});
```
