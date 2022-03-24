#[macro_export]
macro_rules! debug_log {
  (target: $target:expr, $lvl:expr, $($arg:tt)+) => (
    if cfg!(debug_assertions) {
        log!($target, $lvl, $($arg)+);
    }
  );
  ($lvl:expr, $($arg:tt)+) => (
    if cfg!(debug_assertions) {
      log!($lvl, $($arg)+);
    }
  );
}

#[macro_export]
macro_rules! debug_info {
  (target: $target:expr, $($arg:tt)+) => (
    if cfg!(debug_assertions) {
      info!($target, $($arg)+);
    }
  );
  ($($arg:tt)+) => (
    if cfg!(debug_assertions) {
      info!($($arg)+);
    }
  );
}

#[macro_export]
macro_rules! debug_trace {
  (target: $target:expr, $($arg:tt)+) => (
    if cfg!(debug_assertions) {
      trace!($target, $($arg)+);
    }
  );
  ($($arg:tt)+) => (
    if cfg!(debug_assertions) {
      trace!($($arg)+);
    }
  );
}

#[macro_export]
macro_rules! debug_warn {
  (target: $target:expr, $($arg:tt)+) => (
    if cfg!(debug_assertions) {
      warn!($target, $($arg)+);
    }
  );
  ($($arg:tt)+) => (
    if cfg!(debug_assertions) {
      warn!($($arg)+);
    }
  );
}

#[macro_export]
macro_rules! debug_debug {
  (target: $target:expr, $($arg:tt)+) => (
    if cfg!(debug_assertions) {
      debug!($target, $($arg)+);
    }
  );
  ($($arg:tt)+) => (
    if cfg!(debug_assertions) {
      debug!($($arg)+);
    }
  );
}

#[macro_export]
macro_rules! debug_error {
  (target: $target:expr, $($arg:tt)+) => (
    if cfg!(debug_assertions) {
      error!($target, $($arg)+);
    }
  );
  ($($arg:tt)+) => (
    if cfg!(debug_assertions) {
      error!($($arg)+);
    }
  );
}

#[macro_export]
macro_rules! debug_print {
  () => {
    if cfg!(debug_assertions) {
      print!();
    }
  };
  ($($arg:tt)*) => {
    if cfg!(debug_assertions) {
      print!($($arg)*);
    }
  }
}

#[macro_export]
macro_rules! debug_println {
  () => {
    if cfg!(debug_assertions) {
      println!();
    }
  };
  ($($arg:tt)*) => {
    if cfg!(debug_assertions) {
      println!($($arg)*);
    }
  }
}
