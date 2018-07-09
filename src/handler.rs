use std::rc::Rc;

use std::{sync, thread, time};
use std::sync::{Arc, Mutex, Condvar, atomic::{AtomicBool, Ordering}};

use common::RawFunc;
use sync as fpSync;
use sync::{Queue};

pub trait Handler {
    fn is_started(&mut self) -> bool;
    fn is_alive(&mut self) -> bool;

    fn start(&mut self);
    fn stop(&mut self);

    fn post(&mut self, func : RawFunc);
}

#[derive(Clone)]
pub struct HandlerThread {
    started: Arc<Mutex<AtomicBool>>,
    alive: Arc<Mutex<AtomicBool>>,

    inner: Arc<HandlerThreadInner>,

    handle: Arc<Option<thread::JoinHandle<()>>>,
}

impl HandlerThread {
    pub fn new() -> Arc<HandlerThread> {
        return Arc::new(HandlerThread {
            started: Arc::new(Mutex::new(AtomicBool::new(false))),
            alive: Arc::new(Mutex::new(AtomicBool::new(false))),
            inner: Arc::new(HandlerThreadInner::new()),

            handle: Arc::new(None),
        });
    }
}

impl Handler for HandlerThread {

    fn is_started(&mut self) -> bool {
        let _started = self.started.clone();
        let started = _started.lock().unwrap();
        return started.load(Ordering::SeqCst);
    }

    fn is_alive(&mut self) -> bool {
        let _alive = self.alive.clone();
        let alive = _alive.lock().unwrap();
        return alive.load(Ordering::SeqCst);
    }

    fn start(&mut self) {

        {
            let _started = self.started.clone();
            let started = _started.lock().unwrap();
            if started.load(Ordering::SeqCst) {
                return;
            }
            started.store(true, Ordering::SeqCst);

            let _alive = self.alive.clone();
            let alive = _alive.lock().unwrap();
            if alive.load(Ordering::SeqCst) {
                return;
            }
            alive.store(true, Ordering::SeqCst);
        }

        let mut _inner = self.inner.clone();

        self.handle = Arc::new(Some(thread::spawn(move || {

            /*
            let inner : &mut HandlerThreadInner;
            let inner_temp = Arc::get_mut(&mut _inner);
            loop {
                match inner_temp {
                    Some(_x) => {
                        inner = _x;
                        // println!("True");
                        break;
                        },
                    None => {
                        println!("False");
                        continue;
                    }
                }
            }
            */
            Arc::make_mut(&mut _inner).start();

            println!("True");
            // inner.start();
        })));
    }

    fn stop(&mut self) {
        {
            let _started = self.started.clone();
            let started = _started.lock().unwrap();
            if !started.load(Ordering::SeqCst) {
                return;
            }

            let _alive = self.alive.clone();
            let alive = _alive.lock().unwrap();
            if !alive.load(Ordering::SeqCst) {
                return;
            }
            alive.store(false, Ordering::SeqCst);
        }

        if !self.is_alive() {
            return;
        }
        Arc::make_mut(&mut self.inner).stop();

        let mut _handle = Box::new(&mut self.handle);
        let handle = Arc::get_mut(&mut _handle).unwrap();
        handle
            .take().expect("Called stop on non-running thread")
            .join().expect("Could not join spawned thread");
    }

    fn post(&mut self, func: RawFunc) {
        Arc::make_mut(&mut self.inner).post(func);
    }
}

#[derive(Clone)]
struct HandlerThreadInner {
    // this: Option<Arc<HandlerThreadInner>>,

    started: Arc<AtomicBool>,
    alive: Arc<AtomicBool>,
    q: Arc<fpSync::BlockingQueue<RawFunc>>,
}

impl HandlerThreadInner {
    pub fn new() -> HandlerThreadInner {
        return HandlerThreadInner {
            started: Arc::new(AtomicBool::new(false)),
            alive: Arc::new(AtomicBool::new(false)),
            q: Arc::new(<fpSync::BlockingQueue<RawFunc>>::new()),
        };
    }

}

impl Handler for HandlerThreadInner {

    fn is_started(&mut self) -> bool {
        return self.started.load(Ordering::SeqCst);
    }

    fn is_alive(&mut self) -> bool {
        return self.alive.load(Ordering::SeqCst);
    }

    fn start(&mut self){
        self.alive.store(true, Ordering::SeqCst);
        let alive = self.alive.clone();

        if self.is_started() {
            return;
        }
        self.started.store(true, Ordering::SeqCst);

        let q = Arc::make_mut(&mut self.q);

        while alive.load(Ordering::SeqCst) {
            let v = q.take();

            v.invoke();
        }
    }

    fn stop(&mut self) {
        self.alive.store(false, Ordering::SeqCst);
    }

    fn post(&mut self, func: RawFunc) {
        let q = Arc::make_mut(&mut self.q);

        q.put(func);
    }
}

#[test]
fn test_handler_new() {

    let mut _h = HandlerThread::new();
    println!("is_alive {:?}", Arc::make_mut(&mut _h).is_alive());
    println!("is_started {:?}", Arc::make_mut(&mut _h).is_started());
    Arc::make_mut(&mut _h).stop();
    println!("is_alive {:?}", Arc::make_mut(&mut _h).is_alive());
    println!("is_started {:?}", Arc::make_mut(&mut _h).is_started());
    // let mut h1 = _h.clone();
    Arc::make_mut(&mut _h).start();
    println!("is_alive {:?}", Arc::make_mut(&mut _h).is_alive());
    println!("is_started {:?}", Arc::make_mut(&mut _h).is_started());

    let pair = Arc::new((Mutex::new(false), Condvar::new()));
    let pair2 = pair.clone();

    // /*
    Arc::make_mut(&mut _h).post(RawFunc::new(move ||{
        println!("Executed !");
        let &(ref lock, ref cvar) = &*pair2;
        let mut started = lock.lock().unwrap();
        *started = true;

        cvar.notify_one();
        }));
    println!("Test");

    thread::sleep(time::Duration::from_millis(1000));

    println!("is_alive {:?}", Arc::make_mut(&mut _h).is_alive());
    println!("is_started {:?}", Arc::make_mut(&mut _h).is_started());

    Arc::make_mut(&mut _h).stop();

    println!("is_alive {:?}", Arc::make_mut(&mut _h).is_alive());
    println!("is_started {:?}", Arc::make_mut(&mut _h).is_started());

    /*

    let &(ref lock, ref cvar) = &*pair;
    let mut started = lock.lock().unwrap();
    while !*started {
        started = cvar.wait(started).unwrap();
    }
    // */
}
