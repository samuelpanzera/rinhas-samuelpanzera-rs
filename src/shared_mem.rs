use chrono::{DateTime, Utc};
use memmap2::{MmapMut, MmapOptions};
use std::env;
use std::fs::OpenOptions;
use std::io;
use std::sync::atomic::{AtomicU64, Ordering};
use uuid::Uuid;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PaymentRecord {
    pub amount_cents: u16,
    pub is_default_processor: u8,
    pub timestamp_secs: u64,
    pub timestamp_nanos: u32,
}

#[repr(C)]
pub struct PaymentSummaryShared {
    pub default_total_requests: AtomicU64,
    pub default_total_amount_cents: AtomicU64,
    pub fallback_total_requests: AtomicU64,
    pub fallback_total_amount_cents: AtomicU64,
    pub processed_count: AtomicU64,
    pub processed_ids: [AtomicU64; 20000],
    pub record_count: AtomicU64,
    pub records: [PaymentRecord; 50000],
}

impl PaymentSummaryShared {
    pub fn new() -> Self {
        Self {
            default_total_requests: AtomicU64::new(0),
            default_total_amount_cents: AtomicU64::new(0),
            fallback_total_requests: AtomicU64::new(0),
            fallback_total_amount_cents: AtomicU64::new(0),
            processed_count: AtomicU64::new(0),
            processed_ids: std::array::from_fn(|_| AtomicU64::new(0)),
            record_count: AtomicU64::new(0),
            records: std::array::from_fn(|_| PaymentRecord {
                amount_cents: 0,
                is_default_processor: 0,
                timestamp_secs: 0,
                timestamp_nanos: 0,
            }),
        }
    }

    #[inline]
    pub fn add_payment(&self, amount_cents: u16, is_default_processor: bool) {
        if is_default_processor {
            self.default_total_requests.fetch_add(1, Ordering::Relaxed);
            self.default_total_amount_cents
                .fetch_add(amount_cents as u64, Ordering::Relaxed);
        } else {
            self.fallback_total_requests.fetch_add(1, Ordering::Relaxed);
            self.fallback_total_amount_cents
                .fetch_add(amount_cents as u64, Ordering::Relaxed);
        }
    }

    #[inline]
    pub fn read_summary(&self) -> (u64, u64, u64, u64) {
        (
            self.default_total_requests.load(Ordering::Relaxed),
            self.default_total_amount_cents.load(Ordering::Relaxed),
            self.fallback_total_requests.load(Ordering::Relaxed),
            self.fallback_total_amount_cents.load(Ordering::Relaxed),
        )
    }

    #[inline]
    pub fn add_payment_record(
        &self,
        amount_cents: u16,
        is_default_processor: bool,
        timestamp: DateTime<Utc>,
    ) {
        let record = PaymentRecord {
            amount_cents,
            is_default_processor: is_default_processor as u8,
            timestamp_secs: timestamp.timestamp() as u64,
            timestamp_nanos: timestamp.timestamp_subsec_nanos(),
        };

        let idx = (self.record_count.fetch_add(1, Ordering::AcqRel) % 50000) as usize;

        unsafe {
            std::ptr::write(self.records.as_ptr().add(idx) as *mut PaymentRecord, record);
        }
    }
}

pub struct SharedMemoryManager {
    _mmap: MmapMut,
    summary: &'static PaymentSummaryShared,
}

impl SharedMemoryManager {
    pub fn new() -> io::Result<Self> {
        let file_path = Self::get_shared_file_path();

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&file_path)?;

        let size = std::mem::size_of::<PaymentSummaryShared>();

        let current_len = file.metadata()?.len();
        if current_len < size as u64 {
            file.set_len(size as u64)?;
        }

        let mut mmap = unsafe { MmapOptions::new().map_mut(&file)? };

        let summary = unsafe {
            let ptr = mmap.as_mut_ptr() as *mut PaymentSummaryShared;

            if current_len < size as u64 {
                std::ptr::write(ptr, PaymentSummaryShared::new());
            }

            mmap.flush().ok();

            &*ptr
        };

        Ok(Self {
            _mmap: mmap,
            summary,
        })
    }

    fn get_shared_file_path() -> String {
        if let Ok(shared_path) = env::var("SHARED_MEMORY_PATH") {
            format!("{}/payment_summary.dat", shared_path)
        } else {
            if cfg!(windows) {
                let temp_dir = env::temp_dir();
                format!("{}/payment_summary.dat", temp_dir.to_string_lossy())
            } else {
                "/tmp/payment_summary.dat".to_string()
            }
        }
    }

    #[inline]
    pub fn add_payment_if_new(
        &self,
        _correlation_id: Uuid,
        amount_cents: u16,
        is_default_processor: bool,
        timestamp: DateTime<Utc>,
    ) -> bool {
        self.summary.add_payment(amount_cents, is_default_processor);
        self.summary
            .add_payment_record(amount_cents, is_default_processor, timestamp);
        true
    }

    #[inline]
    pub fn get_summary(&self) -> (u64, u64, u64, u64) {
        self.summary.read_summary()
    }

    pub fn get_summary_range(
        &self,
        from_str: Option<&str>,
        to_str: Option<&str>,
    ) -> io::Result<(u64, u64, u64, u64)> {
        if from_str.is_none() && to_str.is_none() {
            return Ok(self.get_summary());
        }

        let parse_date = |s: &str, field_name: &str| {
            DateTime::parse_from_rfc3339(s)
                .map(|dt| dt.with_timezone(&Utc))
                .map_err(|e| {
                    io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("Formato inválido para a data '{}': {}", field_name, e),
                    )
                })
        };

        let from_date = from_str.map(|s| parse_date(s, "from")).transpose()?;
        let to_date = to_str.map(|s| parse_date(s, "to")).transpose()?;

        let record_count = self.summary.record_count.load(Ordering::Acquire);
        let mut summary = (0u64, 0u64, 0u64, 0u64);

        for i in 0..std::cmp::min(record_count, 50000) {
            let idx = i as usize;
            let record = unsafe { std::ptr::read(&self.summary.records[idx]) };

            let timestamp =
                DateTime::from_timestamp(record.timestamp_secs as i64, record.timestamp_nanos)
                    .unwrap_or_else(|| Utc::now());

            let from_ok = from_date.map_or(true, |from| timestamp >= from);
            let to_ok = to_date.map_or(true, |to| timestamp <= to);

            if from_ok && to_ok {
                if record.is_default_processor == 1 {
                    summary.0 += 1;
                    summary.1 += record.amount_cents as u64;
                } else {
                    summary.2 += 1;
                    summary.3 += record.amount_cents as u64;
                }
            }
        }

        Ok(summary)
    }
}

impl Clone for SharedMemoryManager {
    fn clone(&self) -> Self {
        let file_path = Self::get_shared_file_path();

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&file_path)
            .expect("Failed to open existing shared memory file for clone");

        let mut mmap = unsafe {
            MmapOptions::new()
                .map_mut(&file)
                .expect("Failed to map existing file")
        };

        let summary = unsafe {
            let ptr = mmap.as_mut_ptr() as *mut PaymentSummaryShared;
            &*ptr
        };

        Self {
            _mmap: mmap,
            summary,
        }
    }
}

unsafe impl Send for SharedMemoryManager {}
unsafe impl Sync for SharedMemoryManager {}
