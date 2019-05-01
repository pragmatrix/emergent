use shh;
use std::io::{Read, Write};
use std::sync::mpsc;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

pub struct Capture {
    thread: Option<JoinHandle<Vec<u8>>>,
    shutdown_sender: mpsc::Sender<()>,
}

impl Drop for Capture {
    fn drop(&mut self) {
        if let Some(thread) = self.thread.take() {
            self.shutdown_sender.send(()).unwrap();
            thread.join().unwrap();
        }
    }
}

impl Capture {
    pub fn stdout() -> Capture {
        fn flush() {
            std::io::stderr().flush().unwrap();
        }
        Self::begin(shh::stdout().unwrap(), &flush)
    }

    pub fn stderr() -> Capture {
        fn flush() {
            std::io::stderr().flush().unwrap();
        }
        Self::begin(shh::stdout().unwrap(), &flush)
    }

    fn begin<R: Read + Send + 'static, FLUSH: Fn() -> () + Send + 'static>(
        mut handle: R,
        flush: FLUSH,
    ) -> Capture {
        let (shutdown_sender, shutdown_receiver) = mpsc::channel();

        let thread = thread::spawn(move || {
            let mut captured = Vec::new();
            let mut read_to_end = || loop {
                let buf = &mut Vec::new();
                handle.read_to_end(buf).unwrap();
                if buf.is_empty() {
                    break;
                }
                captured.append(buf);
            };

            loop {
                if shutdown_receiver
                    .recv_timeout(Duration::from_millis(1))
                    .is_ok()
                {
                    // flush before pulling everything out.
                    flush();
                    read_to_end();
                    // TODO: this might fill in some buffers, or even block?
                    drop(handle);
                    break;
                } else {
                    read_to_end();
                }
            }

            captured
        });

        Capture {
            thread: Some(thread),
            shutdown_sender,
        }
    }

    pub fn end(mut self) -> Vec<u8> {
        if let Some(thread) = self.thread.take() {
            self.shutdown_sender.send(()).unwrap();
            thread.join().unwrap()
        } else {
            panic!("internal error");
        }
    }
}

#[test]
fn test_capture() {
    // note: this test may fail if --test-threads 1 is not set.

    let capture = Capture::stdout();
    // note: we need to write directly to stdout, because the libtest might divert
    // println!
    let str = "Hello World";
    std::io::stdout().write(str.as_bytes()).unwrap();

    let captured = capture.end();
    assert_eq!("Hello World", String::from_utf8(captured).unwrap());
}

#[test]
fn drop_before_capture_no_content() {
    drop(Capture::stdout());
}

#[test]
fn drop_before_capture_with_content() {
    let capture = Capture::stdout();
    let str = "Hello World";
    std::io::stdout().write(str.as_bytes()).unwrap();
    drop(capture);
}
