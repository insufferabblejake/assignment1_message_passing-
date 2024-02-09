use std::{
    sync::{mpsc, Arc},
    thread,
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

    for (i, t) in tasks.into_iter().enumerate() {

        let tx_clone = tx.clone();
        let name = format!("Worker-{}", i);
        let _ = thread::Builder::new().name(name).spawn(move || {
            let task = t.clone();
            let worker = Worker::create_worker(i as u32);
            let result = worker.process_task(task);
            tx_clone.send(result).unwrap();
        });

    }

    // loop through all results
    for received in &rx {
        println!("The main thread received {}", received);
    }

    // Drop the receiver to close off things
    drop(rx);
}
