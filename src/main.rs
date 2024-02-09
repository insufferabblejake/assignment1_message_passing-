use std::{
    sync::{mpsc, Arc},
    thread,
    time::Duration,
};

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
        thread::sleep(Duration::from_millis(rand::random::<u64>() % 100));
        format!(
            "Worker {} processed task {} with payload {}",
            self.id, task.id, task.payload
        )
    }
}

fn main() {
    let num_tasks = 10;
    let mut tasks: Vec<Arc<Task>> = vec![];

    for i in 0..num_tasks {
        let payload = format!("Task-{} payload", i);
        let task = Arc::new(Task::create_task(i, payload.as_str()));
        tasks.push(task);
    }

    // multiple producer, single consumer channel
    // i.e the threads that we spawn are the producers and we are the consumer
    let (tx, rx) = mpsc::channel();

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

    // Without checking for an explicit count
    // we'll be stuck in this loop forever, waiting for more receives.
    // Why does it think there are more receives? Because each thread
    // receives a tx clone, which means there is/are still Senders
    // and those are only going to be dropped at end of scope.
    // Although why that scope continues to linger, I do not know.
    let mut received_count = 0;
    while received_count < num_tasks {
        if let Ok(received) = rx.recv() {
            println!("{}", received);
            received_count += 1;
        } else {
            // The channel has been closed and all messages have been received
            break;
        }
    }

    for handle in handles {
        handle.join().unwrap();
    }
}
