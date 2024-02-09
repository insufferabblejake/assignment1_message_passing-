use std::{
    sync::{mpsc, Arc},
    thread,
    time::{Duration, Instant},
};

use rand::Rng;

struct Task {
    id: u32,
    payload: String,
}

impl Task {
    fn create_task(id: u32, payload: &str) -> Task {
        Task {
            id,
            payload: payload.to_string(),
        }
    }
}

struct Worker {
    id: u32,
}

impl Worker {
    fn create_worker(id: u32) -> Worker {
        Worker { id }
    }

    // Need to wrap the task in Arc so that we can accept it from the main thread
    fn process_task(&self, task: Arc<Task>) -> String {
        // simulate some random workload
        let mut rng = rand::thread_rng();
        let random_duration = Duration::from_millis(rng.gen_range(50..=200));
        thread::sleep(random_duration);

        // Workload done, evaluate to a result payload
        format!(
            "Worker-{}: Task-{}: processed {}",
            self.id, task.id, task.payload
        )
    }
}

// Loosely, the main thread creates a vector of tasks and distributes one task
// per thread it creates. It also sets up a multiple producer, single consumer
// channel to collect results from each of the threads i.e each thread is the
// producer and the main thread is the consumer.

// The tasks vector and tx end of the channel are closed over
// and avialble to clone as needed for each thread.

// With a cloned tx end per thread, each thread does it's work
// and sends us a result that we as the main thread wait for.
fn main() {
    let num_tasks = 10;
    let mut tasks: Vec<Arc<Task>> = vec![];

    // Arbitrary time out
    let timeout = Duration::from_millis(100);

    for i in 0..num_tasks {
        let payload = format!("Payload-{}", i);
        let task = Arc::new(Task::create_task(i, payload.as_str()));
        tasks.push(task);
    }

    // multiple producer, single consumer channel
    // i.e the threads that we spawn are the producers and we are the consumer
    let (tx, rx) = mpsc::channel();

    let start = Instant::now();

    let mut handles = vec![];
    for (i, t) in tasks.into_iter().enumerate() {
        let tx_clone = tx.clone();
        let name = format!("Worker-{}", i);

        let handle = thread::Builder::new()
            .name(name)
            .spawn(move || {
                let task = t.clone();
                let worker = Worker::create_worker(i as u32);
                let result = worker.process_task(task);
                tx_clone.send(result).unwrap();
            })
            .unwrap();

        handles.push(handle);
    }

    // This loop took me a while to figure out ...
    let mut results: Vec<String> = vec![];
    while Instant::now().duration_since(start) < timeout {
        match rx.try_recv() {
            Ok(received) => {
                results.push(received.clone());
                println!("Received: {}", received);
            }
            Err(mpsc::TryRecvError::Empty) => {
                std::thread::sleep(Duration::from_millis(5)); // Prevent busy waiting
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                // All sender handles have been dropped, you can exit the loop
                println!("All tasks have been processed.");
                break;
            }
        }
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let mut successful_payloads = vec![];

    // FIXME: This logic can be way better. For instance we can return a tuple
    // of String and task-id as the result making our life much easier.
    // Check results and print which tasks have timed out
    for result in results {
        if let Some(payload_part) = result
            .split_whitespace()
            .find(|&part| part.starts_with("Payload-"))
        {
            let payload_id_str = payload_part.trim_start_matches("Payload-");
            if let Ok(payload_id) = payload_id_str.parse::<u32>() {
                successful_payloads.push(payload_id);
            }
        }
    }

    successful_payloads.sort_unstable();

    // Check for missing payload ids
    for expected_id in 0..num_tasks {
        if !successful_payloads.contains(&expected_id) {
            println!("Timeout: Task-{}", expected_id);
        }
    }
}
