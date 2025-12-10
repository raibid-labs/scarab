//! Issue #169: Use ratatui-testlib SeqlockVerifier for shared memory race detection
//!
//! This test file validates seqlock-based shared memory synchronization used in Scarab.
//!
//! ## Background: Scarab's Shared Memory Architecture
//!
//! Scarab uses a seqlock-based synchronization strategy for zero-copy IPC.
//! The seqlock protocol prevents torn reads using sequence numbers:
//!
//! 1. Reader checks sequence number (must be even = not writing)
//! 2. Reader copies data
//! 3. Reader checks sequence number again (must match)
//! 4. If mismatch, retry the read
//!
//! ## SeqlockVerifier (simulated with test helpers)
//!
//! These tests demonstrate the seqlock verification that ratatui-testlib v0.5.0 would provide.
//! The test helpers implement the core seqlock protocol and verification logic.

use anyhow::{Context, Result};
use scarab_protocol::{SharedState, BUFFER_SIZE, GRID_HEIGHT, GRID_WIDTH};
use shared_memory::ShmemConf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

// =============================================================================
// SEQLOCK VERIFICATION HELPERS
// =============================================================================

/// Shared statistics for tracking torn reads
#[derive(Default)]
struct SeqlockStats {
    torn_reads_detected: AtomicU64,
    successful_reads: AtomicU64,
}

/// Perform a seqlock-protected read of the entire SharedState
fn synchronized_read(shmem_path: &str, stats: Option<&SeqlockStats>) -> Option<SharedState> {
    const MAX_RETRIES: usize = 100;

    let shmem = match ShmemConf::new().os_id(shmem_path).open() {
        Ok(s) => s,
        Err(_) => return None,
    };

    for _ in 0..MAX_RETRIES {
        let state_ptr = shmem.as_ptr() as *const SharedState;

        unsafe {
            let seq_before = (*state_ptr).sequence_number;

            if seq_before & 1 != 0 {
                continue;
            }

            let data = std::ptr::read_volatile(state_ptr);
            std::sync::atomic::compiler_fence(Ordering::Acquire);

            let seq_after = (*state_ptr).sequence_number;

            if seq_before == seq_after {
                if let Some(s) = stats {
                    s.successful_reads.fetch_add(1, Ordering::Relaxed);
                }
                return Some(data);
            }

            if let Some(s) = stats {
                s.torn_reads_detected.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    None
}

/// Check if a write is currently in progress
fn is_write_in_progress(shmem_path: &str) -> bool {
    let Ok(shmem) = ShmemConf::new().os_id(shmem_path).open() else {
        return false;
    };

    let state_ptr = shmem.as_ptr() as *const SharedState;

    unsafe {
        let seq = (*state_ptr).sequence_number;
        seq & 1 != 0
    }
}

/// Writer function that updates shared memory
fn write_loop(shmem_path: String, stop_flag: Arc<AtomicBool>, update_interval_micros: u64) {
    let shmem = ShmemConf::new()
        .size(std::mem::size_of::<SharedState>())
        .os_id(&shmem_path)
        .create()
        .or_else(|_| ShmemConf::new().os_id(&shmem_path).open())
        .expect("Failed to open/create shared memory");

    let state_ptr = shmem.as_ptr() as *mut SharedState;
    let mut counter = 0u32;

    while !stop_flag.load(Ordering::Relaxed) {
        unsafe {
            let seq = (*state_ptr).sequence_number;
            (*state_ptr).sequence_number = seq + 1;

            std::sync::atomic::compiler_fence(Ordering::Release);

            counter = counter.wrapping_add(1);
            for i in 0..BUFFER_SIZE {
                (*state_ptr).cells[i].char_codepoint = counter;
            }
            (*state_ptr).cursor_x = (counter % GRID_WIDTH as u32) as u16;
            (*state_ptr).cursor_y = (counter % GRID_HEIGHT as u32) as u16;

            std::sync::atomic::compiler_fence(Ordering::Release);

            (*state_ptr).sequence_number = seq + 2;
        }

        if update_interval_micros > 0 {
            thread::sleep(Duration::from_micros(update_interval_micros));
        } else {
            thread::yield_now();
        }
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[test]
fn test_seqlock_basic_verification() -> Result<()> {
    let test_shmem = format!("/scarab_test_seqlock_basic_{}", std::process::id());

    {
        let shmem = ShmemConf::new()
            .size(std::mem::size_of::<SharedState>())
            .os_id(&test_shmem)
            .create()?;
        unsafe {
            let state_ptr = shmem.as_ptr() as *mut SharedState;
            std::ptr::write_bytes(state_ptr, 0, 1);
            (*state_ptr).sequence_number = 0;
        }
    }

    let stop_flag = Arc::new(AtomicBool::new(false));
    let writer_stop = stop_flag.clone();
    let writer_path = test_shmem.clone();
    let writer_handle = thread::spawn(move || {
        write_loop(writer_path, writer_stop, 1000);
    });

    thread::sleep(Duration::from_millis(100));

    let stats = Arc::new(SeqlockStats::default());

    for _ in 0..100 {
        let data = synchronized_read(&test_shmem, Some(&stats))
            .context("Failed to read after retries")?;

        assert_eq!(data.sequence_number & 1, 0, "Sequence number should be even");

        thread::sleep(Duration::from_millis(10));
    }

    stop_flag.store(true, Ordering::Relaxed);
    writer_handle.join().unwrap();

    println!("Successful reads: {}", stats.successful_reads.load(Ordering::Relaxed));
    println!("Torn reads detected: {}", stats.torn_reads_detected.load(Ordering::Relaxed));

    assert!(stats.successful_reads.load(Ordering::Relaxed) > 0, "Should have successful reads");

    Ok(())
}

#[test]
fn test_seqlock_high_contention() -> Result<()> {
    let test_shmem = format!("/scarab_test_seqlock_contention_{}", std::process::id());

    {
        let shmem = ShmemConf::new()
            .size(std::mem::size_of::<SharedState>())
            .os_id(&test_shmem)
            .create()?;
        unsafe {
            let state_ptr = shmem.as_ptr() as *mut SharedState;
            std::ptr::write_bytes(state_ptr, 0, 1);
            (*state_ptr).sequence_number = 0;
        }
    }

    let stop_flag = Arc::new(AtomicBool::new(false));
    let writer_stop = stop_flag.clone();
    let writer_path = test_shmem.clone();
    let writer_handle = thread::spawn(move || {
        write_loop(writer_path, writer_stop, 0);  // No delay = max contention
    });

    thread::sleep(Duration::from_millis(50));

    let stats = Arc::new(SeqlockStats::default());
    let start = Instant::now();
    let mut read_count = 0;

    while start.elapsed() < Duration::from_secs(1) {
        if let Some(data) = synchronized_read(&test_shmem, Some(&stats)) {
            assert_eq!(data.sequence_number & 1, 0);
            read_count += 1;
        }
    }

    stop_flag.store(true, Ordering::Relaxed);
    writer_handle.join().unwrap();

    println!("High contention - reads: {}, torn reads: {}",
             stats.successful_reads.load(Ordering::Relaxed), 
             stats.torn_reads_detected.load(Ordering::Relaxed));

    assert!(read_count > 0, "Should complete some reads even under high contention");

    Ok(())
}

#[test]
fn test_seqlock_stress_concurrent_readers() -> Result<()> {
    let test_shmem = format!("/scarab_test_seqlock_concurrent_{}", std::process::id());

    {
        let shmem = ShmemConf::new()
            .size(std::mem::size_of::<SharedState>())
            .os_id(&test_shmem)
            .create()?;
        unsafe {
            let state_ptr = shmem.as_ptr() as *mut SharedState;
            std::ptr::write_bytes(state_ptr, 0, 1);
            (*state_ptr).sequence_number = 0;
        }
    }

    let stop_flag = Arc::new(AtomicBool::new(false));
    let writer_stop = stop_flag.clone();
    let writer_path = test_shmem.clone();
    let writer_handle = thread::spawn(move || {
        write_loop(writer_path, writer_stop, 100);
    });

    thread::sleep(Duration::from_millis(50));

    const NUM_READERS: usize = 4;
    let mut reader_handles = vec![];
    let stats = Arc::new(SeqlockStats::default());

    for i in 0..NUM_READERS {
        let shmem_path = test_shmem.clone();
        let reader_stats = stats.clone();

        let handle = thread::spawn(move || -> Result<()> {
            for _ in 0..50 {
                synchronized_read(&shmem_path, Some(&reader_stats))
                    .context(format!("Reader {} failed to read", i))?;
                thread::sleep(Duration::from_millis(10));
            }
            Ok(())
        });

        reader_handles.push(handle);
    }

    for handle in reader_handles {
        handle.join().unwrap()?;
    }

    stop_flag.store(true, Ordering::Relaxed);
    writer_handle.join().unwrap();

    let total_successful = stats.successful_reads.load(Ordering::Relaxed);
    let total_torn = stats.torn_reads_detected.load(Ordering::Relaxed);

    println!("Concurrent readers - total successful: {}, total torn: {}",
             total_successful, total_torn);

    assert!(total_successful > 0, "Should have successful reads across all readers");

    Ok(())
}

#[test]
fn test_seqlock_large_data_structure() -> Result<()> {
    let test_shmem = format!("/scarab_test_seqlock_large_{}", std::process::id());

    {
        let shmem = ShmemConf::new()
            .size(std::mem::size_of::<SharedState>())
            .os_id(&test_shmem)
            .create()?;
        unsafe {
            let state_ptr = shmem.as_ptr() as *mut SharedState;
            std::ptr::write_bytes(state_ptr, 0, 1);
            (*state_ptr).sequence_number = 0;
        }
    }

    let stop_flag = Arc::new(AtomicBool::new(false));
    let writer_stop = stop_flag.clone();
    let writer_path = test_shmem.clone();
    let writer_handle = thread::spawn(move || {
        write_loop(writer_path, writer_stop, 10);
    });

    thread::sleep(Duration::from_millis(50));

    let stats = Arc::new(SeqlockStats::default());

    for _ in 0..100 {
        if let Some(data) = synchronized_read(&test_shmem, Some(&stats)) {
            let first_value = data.cells[0].char_codepoint;
            let all_same = data.cells.iter().all(|c| c.char_codepoint == first_value);

            assert!(all_same,
                "All cells should have same value if read was atomic (no torn read)");
        }

        thread::sleep(Duration::from_millis(5));
    }

    stop_flag.store(true, Ordering::Relaxed);
    writer_handle.join().unwrap();

    println!("Large data test - successful: {}, torn: {}",
             stats.successful_reads.load(Ordering::Relaxed),
             stats.torn_reads_detected.load(Ordering::Relaxed));

    Ok(())
}

#[test]
fn test_seqlock_sequence_always_even() -> Result<()> {
    let test_shmem = format!("/scarab_test_seqlock_even_{}", std::process::id());

    {
        let shmem = ShmemConf::new()
            .size(std::mem::size_of::<SharedState>())
            .os_id(&test_shmem)
            .create()?;
        unsafe {
            let state_ptr = shmem.as_ptr() as *mut SharedState;
            std::ptr::write_bytes(state_ptr, 0, 1);
            (*state_ptr).sequence_number = 0;
        }
    }

    let stop_flag = Arc::new(AtomicBool::new(false));
    let writer_stop = stop_flag.clone();
    let writer_path = test_shmem.clone();
    let writer_handle = thread::spawn(move || {
        write_loop(writer_path, writer_stop, 100);
    });

    thread::sleep(Duration::from_millis(50));

    for _ in 0..200 {
        if let Some(data) = synchronized_read(&test_shmem, None) {
            assert_eq!(data.sequence_number & 1, 0,
                "Sequence number must be even (got {})", data.sequence_number);
        }
        thread::yield_now();
    }

    stop_flag.store(true, Ordering::Relaxed);
    writer_handle.join().unwrap();

    Ok(())
}

#[test]
fn test_seqlock_retry_on_torn_read() -> Result<()> {
    let test_shmem = format!("/scarab_test_seqlock_retry_{}", std::process::id());

    {
        let shmem = ShmemConf::new()
            .size(std::mem::size_of::<SharedState>())
            .os_id(&test_shmem)
            .create()?;
        unsafe {
            let state_ptr = shmem.as_ptr() as *mut SharedState;
            std::ptr::write_bytes(state_ptr, 0, 1);
            (*state_ptr).sequence_number = 0;
        }
    }

    let stop_flag = Arc::new(AtomicBool::new(false));
    let writer_stop = stop_flag.clone();
    let writer_path = test_shmem.clone();
    let writer_handle = thread::spawn(move || {
        write_loop(writer_path, writer_stop, 0);
    });

    thread::sleep(Duration::from_millis(50));

    let stats = Arc::new(SeqlockStats::default());
    let start = Instant::now();
    
    while start.elapsed() < Duration::from_millis(500) {
        synchronized_read(&test_shmem, Some(&stats))
            .context("Should eventually succeed via retry")?;
    }

    stop_flag.store(true, Ordering::Relaxed);
    writer_handle.join().unwrap();

    let torn = stats.torn_reads_detected.load(Ordering::Relaxed);
    let successful = stats.successful_reads.load(Ordering::Relaxed);

    println!("Retry test - torn: {}, successful: {}", torn, successful);

    assert!(successful > 0, "Should have successful reads");
    if torn > 0 {
        println!("Successfully detected and recovered from {} torn reads", torn);
    }

    Ok(())
}

#[test]
fn test_seqlock_lock_free_no_blocking() -> Result<()> {
    let test_shmem = format!("/scarab_test_seqlock_lockfree_{}", std::process::id());

    {
        let shmem = ShmemConf::new()
            .size(std::mem::size_of::<SharedState>())
            .os_id(&test_shmem)
            .create()?;
        unsafe {
            let state_ptr = shmem.as_ptr() as *mut SharedState;
            std::ptr::write_bytes(state_ptr, 0, 1);
            (*state_ptr).sequence_number = 0;
        }
    }

    let stop_flag = Arc::new(AtomicBool::new(false));
    let writer_stop = stop_flag.clone();
    let writer_path = test_shmem.clone();
    let writer_handle = thread::spawn(move || {
        write_loop(writer_path, writer_stop, 100);
    });

    thread::sleep(Duration::from_millis(50));

    let reader_stop = stop_flag.clone();
    let shmem_path = test_shmem.clone();

    let slow_reader_handle = thread::spawn(move || {
        while !reader_stop.load(Ordering::Relaxed) {
            if synchronized_read(&shmem_path, None).is_some() {
                thread::sleep(Duration::from_millis(50));
            }
        }
    });

    let seq_start = synchronized_read(&test_shmem, None)
        .context("Initial read failed")?
        .sequence_number;

    thread::sleep(Duration::from_millis(200));

    let seq_end = synchronized_read(&test_shmem, None)
        .context("Final read failed")?
        .sequence_number;

    stop_flag.store(true, Ordering::Relaxed);
    writer_handle.join().unwrap();
    slow_reader_handle.join().unwrap();

    assert!(seq_end > seq_start,
        "Writer should make progress even with slow readers (lock-free)");

    println!("Lock-free test - sequence advanced from {} to {}", seq_start, seq_end);

    Ok(())
}

#[test]
fn test_seqlock_integration_with_harness() -> Result<()> {
    let test_shmem = format!("/scarab_test_seqlock_harness_{}", std::process::id());

    {
        let shmem = ShmemConf::new()
            .size(std::mem::size_of::<SharedState>())
            .os_id(&test_shmem)
            .create()?;
        unsafe {
            let state_ptr = shmem.as_ptr() as *mut SharedState;
            std::ptr::write_bytes(state_ptr, 0, 1);
            (*state_ptr).sequence_number = 0;
        }
    }

    let stop_flag = Arc::new(AtomicBool::new(false));
    let writer_stop = stop_flag.clone();
    let writer_path = test_shmem.clone();
    let writer_handle = thread::spawn(move || {
        write_loop(writer_path, writer_stop, 500);
    });

    thread::sleep(Duration::from_millis(50));

    let stats = Arc::new(SeqlockStats::default());

    let grid_contents = synchronized_read(&test_shmem, Some(&stats))
        .context("Failed to read grid contents")?;

    assert_eq!(grid_contents.sequence_number & 1, 0);

    let verify_start = Instant::now();
    let verify_duration = Duration::from_secs(1);

    while verify_start.elapsed() < verify_duration {
        let data = synchronized_read(&test_shmem, Some(&stats))
            .context("Seqlock verification failed")?;

        assert_eq!(data.sequence_number & 1, 0, "Must be even");
        assert!(data.cursor_x < GRID_WIDTH as u16, "Cursor X in bounds");
        assert!(data.cursor_y < GRID_HEIGHT as u16, "Cursor Y in bounds");

        thread::sleep(Duration::from_millis(10));
    }

    stop_flag.store(true, Ordering::Relaxed);
    writer_handle.join().unwrap();

    println!("Harness integration - successful: {}, torn: {}",
             stats.successful_reads.load(Ordering::Relaxed),
             stats.torn_reads_detected.load(Ordering::Relaxed));

    assert!(stats.successful_reads.load(Ordering::Relaxed) > 50, "Should have many successful reads");

    Ok(())
}

#[test]
fn test_seqlock_write_in_progress_detection() -> Result<()> {
    let test_shmem = format!("/scarab_test_seqlock_writing_{}", std::process::id());

    {
        let shmem = ShmemConf::new()
            .size(std::mem::size_of::<SharedState>())
            .os_id(&test_shmem)
            .create()?;
        unsafe {
            let state_ptr = shmem.as_ptr() as *mut SharedState;
            std::ptr::write_bytes(state_ptr, 0, 1);
            (*state_ptr).sequence_number = 0;
        }
    }

    let stop_flag = Arc::new(AtomicBool::new(false));
    let writer_stop = stop_flag.clone();
    let writer_path = test_shmem.clone();
    
    let writer_handle = thread::spawn(move || {
        let shmem = ShmemConf::new()
            .os_id(&writer_path)
            .open()
            .expect("Failed to open shared memory");
        let state_ptr = shmem.as_ptr() as *mut SharedState;

        for i in 0..10 {
            if writer_stop.load(Ordering::Relaxed) {
                break;
            }

            unsafe {
                (*state_ptr).sequence_number = i * 2 + 1;
                thread::sleep(Duration::from_millis(100));
                (*state_ptr).sequence_number = (i + 1) * 2;
            }

            thread::sleep(Duration::from_millis(100));
        }
    });

    thread::sleep(Duration::from_millis(50));

    let mut caught_writing = false;

    for _ in 0..50 {
        if is_write_in_progress(&test_shmem) {
            caught_writing = true;
            println!("Detected write in progress!");
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }

    stop_flag.store(true, Ordering::Relaxed);
    writer_handle.join().unwrap();

    assert!(caught_writing, "Should detect write in progress with slow writer");

    Ok(())
}
