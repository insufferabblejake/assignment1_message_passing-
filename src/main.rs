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
    // Create the task in main thread
    let task = Task::create_task(1, "some work");
    // wrap it in Arc to send to the worker thread
    let task = Arc::new(task);

    // tx is the transmitter
    // rx is the receiver.
    // tx is closed over by the thread API
    // and we use that to send some value from *that * thread
    // to the main thread
    // remember that mpsc is multiple producer and single consumer
    // in our case it is multiple worker threads, send stuff to
    // the main thread
    let (tx, rx) = mpsc::channel();

    // the thread we spawn has "closed over" local vars(?) such as task, tx etc.
    let name = String::from("worker");
    let _ = thread::Builder::new().name(name).spawn(move || {
        let thread_task = task.clone();
        // let's create a worker here
        let worker = Worker::create_worker(1);
        let result = worker.process_task(thread_task);
        tx.send(result).unwrap();
    });

    let received = rx.recv().unwrap();
    println!("The main thread received {}", received);
}
