use memmap2::{MmapMut, MmapOptions};
use std::collections::HashSet;
use std::env;
use std::fs::OpenOptions;
use std::io;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use uuid::Uuid;


#[repr(C)]
pub struct PaymentSummaryShared {
    pub default_total_requests: AtomicU64,
    pub default_total_amount_cents: AtomicU64,
    pub fallback_total_requests: AtomicU64,
    pub fallback_total_amount_cents: AtomicU64,
    pub processed_count: AtomicU64,
    pub processed_ids: [AtomicU64; 20000],
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
    pub fn is_correlation_id_processed(&self, correlation_id: &Uuid) -> bool {
        let uuid_bytes = correlation_id.as_u128();
        let high = (uuid_bytes >> 64) as u64;
        let low = uuid_bytes as u64;
        
        let count = self.processed_count.load(Ordering::Acquire);
        let search_limit = std::cmp::min(count, 10000);
        
        for i in 0..search_limit {
            let position_to_check = count.saturating_sub(1 + i);
            let idx = (position_to_check % 10000) * 2;
            let stored_high = self.processed_ids[idx as usize].load(Ordering::Acquire);
            let stored_low = self.processed_ids[(idx + 1) as usize].load(Ordering::Acquire);
            
            if stored_high == high && stored_low == low {
                return true;
            }
        }
        
        false
    }
  
    #[inline]
    pub fn mark_correlation_id_processed(&self, correlation_id: &Uuid) -> bool {
        let uuid_bytes = correlation_id.as_u128();
        let high = (uuid_bytes >> 64) as u64;
        let low = uuid_bytes as u64;
        let count = self.processed_count.load(Ordering::Acquire);
        let search_limit = std::cmp::min(count, 10000);
        
        for i in 0..search_limit {
            let position_to_check = count.saturating_sub(1 + i);
            let idx = (position_to_check % 10000) * 2;
            let stored_high = self.processed_ids[idx as usize].load(Ordering::Acquire);
            let stored_low = self.processed_ids[(idx + 1) as usize].load(Ordering::Acquire);
            
            if stored_high == high && stored_low == low {
                return false;
            }
        }
        let position = self.processed_count.fetch_add(1, Ordering::AcqRel);
        let idx = (position % 10000) * 2;
        
        let existing_high = self.processed_ids[idx as usize].load(Ordering::Acquire);
        let existing_low = self.processed_ids[(idx + 1) as usize].load(Ordering::Acquire);
        
        if existing_high != 0 || existing_low != 0 {
            if existing_high == high && existing_low == low {
                return false;
            }
        }
        
        self.processed_ids[idx as usize].store(high, Ordering::Release);
        self.processed_ids[(idx + 1) as usize].store(low, Ordering::Release);
        
        true
    }
}

pub struct SharedMemoryManager {
    _mmap: MmapMut,
    summary: &'static PaymentSummaryShared,
    processed_payments: Mutex<HashSet<Uuid>>,
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
            processed_payments: Mutex::new(HashSet::new()),
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

    pub fn is_payment_already_processed(&self, correlation_id: &Uuid) -> bool {
        {
            let processed = self.processed_payments.lock().unwrap();
            if processed.contains(correlation_id) {
                return true;
            }
        }
        
        self.summary.is_correlation_id_processed(correlation_id)
    }


    #[inline]
    pub fn add_payment_if_new(
        &self,
        correlation_id: Uuid,
        amount_cents: u16,
        is_default_processor: bool,
    ) -> bool {
        if !self.summary.mark_correlation_id_processed(&correlation_id) {
            return false;
        }
        
        {
            let mut processed = self.processed_payments.lock().unwrap();
            processed.insert(correlation_id);
        }
        
        self.summary.add_payment(amount_cents, is_default_processor);
        
        true
    }

    #[inline]
    pub fn get_summary(&self) -> (u64, u64, u64, u64) {
        self.summary.read_summary()
    }

    pub fn get_summary_range(
        &self,
        _from_str: Option<&str>,
        _to_str: Option<&str>,
    ) -> io::Result<(u64, u64, u64, u64)> {
        Ok(self.get_summary())
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
            processed_payments: Mutex::new(HashSet::new()),
        }
    }
}

unsafe impl Send for SharedMemoryManager {}
unsafe impl Sync for SharedMemoryManager {}
