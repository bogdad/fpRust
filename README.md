# fpRust

[![tag](https://img.shields.io/github/tag/TeaEntityLab/fpRust.svg)](https://github.com/TeaEntityLab/fpRust)

[![license](https://img.shields.io/github/license/TeaEntityLab/fpRust.svg?style=social&label=License)](https://github.com/TeaEntityLab/fpRust)
[![stars](https://img.shields.io/github/stars/TeaEntityLab/fpRust.svg?style=social&label=Stars)](https://github.com/TeaEntityLab/fpRust)
[![forks](https://img.shields.io/github/forks/TeaEntityLab/fpRust.svg?style=social&label=Fork)](https://github.com/TeaEntityLab/fpRust)

Monad, Functional Programming features for Rust

# Why

I love functional programing, Rx-style coding.

However it's hard to implement them in Rust, and there're few libraries to achieve parts of them.

Thus I implemented fpRust. I hope you would like it :)

# Features

* Optional/Maybe (a wrapper to built-in Option<T>, to make it more like a monad version *`Maybe`*)

* Monad, Rx-like

* Publisher

* Fp functions
  * Currently only compose!() <- __macro__

~~* Pattern matching~~

~~* PythonicGenerator-like Coroutine(yield/yieldFrom)~~


# Usage

## Optional (IsPresent/IsNil, Or, Let)

```rust
extern crate fp_rust;

use fp_rust::maybe::Maybe;

// fmap & map (sync)
Maybe::val(true).fmap(|x| {return Maybe::val(!x.unwrap())}).unwrap(); // false
Maybe::val(false).fmap(|x| {return Maybe::val(!x.unwrap())}).unwrap(); // true

Maybe::val(true).map(|x| {return Some(!x.unwrap())}).unwrap(); // false
Maybe::val(false).map(|x| {return Some(!x.unwrap())}).unwrap(); // true

// fantasy-land: Apply ap()
Maybe::val(1).ap(
   Maybe::val(|x: Option<i16>| {
       if x.unwrap() > 0 {
           return Some(true)
       } else {
           return Some(false)
       }
   })
).unwrap(); // true

// or
Maybe::just(None::<bool>).or(false); // false
Maybe::val(true).or(false); // true

// unwrap
Maybe::val(true).unwrap(); //true

use std::panic;
let none_unwrap = panic::catch_unwind(|| {
    Maybe::just(None::<bool>).unwrap();
});
none_unwrap.is_err(); //true

// Get raw Option<T>
let v = match Maybe::val(true).option() {
    None => false,
    Some(_x) => true,
}; // true
let v = match Maybe::just(None::<bool>).option() {
    None => false,
    Some(_x) => true,
}; // false
```

## MonadIO (RxObserver-like)

Example:
```rust

extern crate fp_rust;

use std::{
    thread,
    time,
    sync::{
        Arc,
        Mutex,
        Condvar,
    }
};

use fp_rust::handler::{
    Handler,
    HandlerThread,
};
use fp_rust::common::SubscriptionFunc;
use fp_rust::monadio::{
    MonadIO,
    of,
};

// fmap & map (sync)
let mut _subscription = Arc::new(SubscriptionFunc::new(move |x: Arc<u16>| {
    println!("monadioSync {:?}", x); // monadioSync 36
    assert_eq!(36, *Arc::make_mut(&mut x.clone()));
}));
let mut subscription = _subscription.clone();
let monadioSync = MonadIO::just(1)
    .fmap(|x| MonadIO::new(move || x * 4))
    .map(|x| x * 3)
    .map(|x| x * 3);
monadioSync.subscribe(subscription);

// fmap & map (async)
let mut _handlerObserveOn = HandlerThread::new_with_mutex();
let mut _handlerSubscribeOn = HandlerThread::new_with_mutex();
let monadioAsync = MonadIO::new_with_handlers(
    || {
        println!("In string");
        String::from("ok")
    },
    Some(_handlerObserveOn.clone()),
    Some(_handlerSubscribeOn.clone()),
);

let pair = Arc::new((Mutex::new(false), Condvar::new()));
let pair2 = pair.clone();

thread::sleep(time::Duration::from_millis(100));

let subscription = Arc::new(SubscriptionFunc::new(move |x: Arc<String>| {
    println!("monadioAsync {:?}", x); // monadioAsync ok

    let &(ref lock, ref cvar) = &*pair2;
    let mut started = lock.lock().unwrap();
    *started = true;

    cvar.notify_one(); // Unlock here
}));
monadioAsync.subscribe(subscription);
{
    let mut handlerObserveOn = _handlerObserveOn.lock().unwrap();
    let mut handlerSubscribeOn = _handlerSubscribeOn.lock().unwrap();

    println!("hh2");
    handlerObserveOn.start();
    handlerSubscribeOn.start();
    println!("hh2 running");

    handlerObserveOn.post(RawFunc::new(move || {}));
    handlerObserveOn.post(RawFunc::new(move || {}));
    handlerObserveOn.post(RawFunc::new(move || {}));
    handlerObserveOn.post(RawFunc::new(move || {}));
    handlerObserveOn.post(RawFunc::new(move || {}));
}
thread::sleep(time::Duration::from_millis(100));

let &(ref lock, ref cvar) = &*pair;
let mut started = lock.lock().unwrap();
// Waiting for being unlcoked
while !*started {
    started = cvar.wait(started).unwrap();
}
```

## Compose

Example:

```rust
#[macro_use]
extern crate fp_rust

use fp_rust::fp::compose_two;

let add = |x| x + 2;
let multiply = |x| x * 2;
let divide = |x| x / 2;
let intermediate = compose!(add, multiply, divide);

println!("Value: {}", intermediate(10)); // Value: 12
```
