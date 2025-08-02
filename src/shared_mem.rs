use chrono::{DateTime, Utc};
use memmap2::{MmapMut, MmapOptions};
use std::collections::BTreeMap;
use std::env;
use std::fs::OpenOptions;
use std::io;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

// Estrutura para armazenar pagamentos individuais com timestamp
#[derive(Clone, Debug)]
pub struct PaymentRecord {
    pub amount_cents: u16, // Otimizado: 1990 cents cabe em u16 (75% memory savings)
    pub is_default_processor: bool,
    // Timestamp removido - já está como chave no BTreeMap
}

// Estrutura ultra-simples - apenas 32 bytes total
#[repr(C)]
pub struct PaymentSummaryShared {
    // Processador padrão
    pub default_total_requests: AtomicU64,
    pub default_total_amount_cents: AtomicU64,

    // Processador fallback
    pub fallback_total_requests: AtomicU64,
    pub fallback_total_amount_cents: AtomicU64,
}

impl PaymentSummaryShared {
    pub fn new() -> Self {
        Self {
            default_total_requests: AtomicU64::new(0),
            default_total_amount_cents: AtomicU64::new(0),
            fallback_total_requests: AtomicU64::new(0),
            fallback_total_amount_cents: AtomicU64::new(0),
        }
    }

    // Incrementa contadores atomicamente - máxima performance
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

    // Lê valores atuais atomicamente
    #[inline]
    pub fn read_summary(&self) -> (u64, u64, u64, u64) {
        (
            self.default_total_requests.load(Ordering::Relaxed),
            self.default_total_amount_cents.load(Ordering::Relaxed),
            self.fallback_total_requests.load(Ordering::Relaxed),
            self.fallback_total_amount_cents.load(Ordering::Relaxed),
        )
    }
}

pub struct SharedMemoryManager {
    _mmap: MmapMut,
    summary: &'static PaymentSummaryShared,
    // BTreeMap ordenado por timestamp para consultas O(log n + k)
    payment_history: Mutex<BTreeMap<DateTime<Utc>, Vec<PaymentRecord>>>,
}

impl SharedMemoryManager {
    pub fn new() -> io::Result<Self> {
        let file_path = Self::get_shared_file_path();

        // ✅ CORREÇÃO: Clear só deve acontecer no início da aplicação, não a cada new()
        // O clear automático foi removido para preservar dados durante requisições

        // Cria ou abre o arquivo para memória compartilhada
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&file_path)?;

        let size = std::mem::size_of::<PaymentSummaryShared>();

        // Define o tamanho do arquivo se necessário
        let current_len = file.metadata()?.len();
        if current_len < size as u64 {
            file.set_len(size as u64)?;
        }

        // Mapeia o arquivo em memória
        let mut mmap = unsafe { MmapOptions::new().map_mut(&file)? };

        let summary = unsafe {
            let ptr = mmap.as_mut_ptr() as *mut PaymentSummaryShared;

            // ✅ CORREÇÃO: Só inicializa se o arquivo está vazio/novo
            if current_len < size as u64 {
                // Arquivo novo - inicializa com dados limpos
                std::ptr::write(ptr, PaymentSummaryShared::new());
                println!("🆕 Initialized new shared memory file");
            } else {
                // Arquivo existente - preserva dados existentes
                println!("📂 Reusing existing shared memory file");
            }

            // Força sincronização para garantir que outros processos vejam
            if let Err(e) = mmap.flush() {
                eprintln!("Warning: Failed to flush mmap: {}", e);
            }

            &*ptr
        };

        println!("🔥 BTreeMap-optimized shared memory initialized with clean state");

        Ok(Self {
            _mmap: mmap,
            summary,
            // BTreeMap sempre inicia vazio para garantir consistência
            payment_history: Mutex::new(BTreeMap::new()),
        })
    }

    fn get_shared_file_path() -> String {
        // Usa o volume compartilhado do Docker se disponível
        if let Ok(shared_path) = env::var("SHARED_MEMORY_PATH") {
            format!("{}/payment_summary.dat", shared_path)
        } else {
            // Fallback para desenvolvimento local (Windows/Linux compatível)
            if cfg!(windows) {
                let temp_dir = env::temp_dir();
                format!("{}/payment_summary.dat", temp_dir.to_string_lossy())
            } else {
                "/tmp/payment_summary.dat".to_string()
            }
        }
    }

    #[inline]
    pub fn add_payment(
        &self,
        amount_cents: u16,
        is_default_processor: bool,
        processed_at: DateTime<Utc>,
    ) {
        // Adiciona ao resumo atômico
        self.summary.add_payment(amount_cents, is_default_processor);

        // Adiciona ao BTreeMap ordenado por timestamp - O(log n)
        let mut history = self.payment_history.lock().unwrap();

        history
            .entry(processed_at)
            .or_insert_with(Vec::new)
            .push(PaymentRecord {
                amount_cents,
                is_default_processor,
            });

        // TODO: Remover depois dos testes - cleanup removido pois pagamentos duram apenas 1 dia
        // Não precisa mais de cleanup de 7 dias
    }

    #[inline]
    pub fn get_summary(&self) -> (u64, u64, u64, u64) {
        self.summary.read_summary()
    }

    // Implementação OTIMIZADA com BTreeMap - O(log n + k) em vez de O(n)
    pub fn get_summary_range(
        &self,
        from_str: Option<&str>,
        to_str: Option<&str>,
    ) -> io::Result<(u64, u64, u64, u64)> {
        // Se não há filtros de data, retorna o resumo completo
        if from_str.is_none() && to_str.is_none() {
            return Ok(self.get_summary());
        }

        // Parse das datas de filtro
        let from_date = if let Some(from) = from_str {
            match DateTime::parse_from_rfc3339(from) {
                Ok(dt) => Some(dt.with_timezone(&Utc)),
                Err(e) => {
                    println!("❌ Failed to parse 'from' date '{}': {}", from, e);
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("Invalid 'from' date format: {}", e),
                    ));
                }
            }
        } else {
            None
        };

        let to_date = if let Some(to) = to_str {
            match DateTime::parse_from_rfc3339(to) {
                Ok(dt) => Some(dt.with_timezone(&Utc)),
                Err(e) => {
                    println!("❌ Failed to parse 'to' date '{}': {}", to, e);
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("Invalid 'to' date format: {}", e),
                    ));
                }
            }
        } else {
            None
        };

        println!(
            "🔍 BTreeMap range query - from: {:?}, to: {:?}",
            from_date, to_date
        );

        // BUSCA BINÁRIA OTIMIZADA - O(log n + k)
        let history = self.payment_history.lock().unwrap();
        let mut default_requests = 0u64;
        let mut default_amount_cents = 0u64;
        let mut fallback_requests = 0u64;
        let mut fallback_amount_cents = 0u64;

        // Cria o range apropriado para busca binária
        let range = match (from_date, to_date) {
            (Some(from), Some(to)) => {
                println!("🎯 Range query: {} to {}", from, to);
                history.range(from..=to)
            }
            (Some(from), None) => {
                println!("🎯 Range query: from {}", from);
                history.range(from..)
            }
            (None, Some(to)) => {
                println!("🎯 Range query: to {}", to);
                history.range(..=to)
            }
            _ => {
                println!("🎯 Full range query");
                history.range(..)
            }
        };

        let mut total_payments_processed = 0u64;

        // Itera apenas sobre os timestamps no período relevante - O(k)
        for (_timestamp, payments_at_time) in range {
            for payment in payments_at_time {
                total_payments_processed += 1;

                if payment.is_default_processor {
                    default_requests += 1;
                    default_amount_cents += payment.amount_cents as u64; // Zero-cost cast
                } else {
                    fallback_requests += 1;
                    fallback_amount_cents += payment.amount_cents as u64; // Zero-cost cast
                }
            }
        }

        println!(
            "🚀 BTreeMap optimized query processed {} payments (vs O(n) with VecDeque)",
            total_payments_processed
        );

        println!(
            "📊 Filtered summary - default: {} requests, {} cents | fallback: {} requests, {} cents",
            default_requests, default_amount_cents, fallback_requests, fallback_amount_cents
        );

        Ok((
            default_requests,
            default_amount_cents,
            fallback_requests,
            fallback_amount_cents,
        ))
    }
}

impl Clone for SharedMemoryManager {
    fn clone(&self) -> Self {
        // ✅ CORREÇÃO: Clone reutiliza o mesmo arquivo sem chamar new() que limparia dados
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

        // Clona o BTreeMap atual para o novo manager
        let cloned_history = self.payment_history.lock().unwrap().clone();

        println!("📋 Cloned SharedMemoryManager preserving existing data");

        Self {
            _mmap: mmap,
            summary,
            payment_history: Mutex::new(cloned_history),
        }
    }
}

unsafe impl Send for SharedMemoryManager {}
unsafe impl Sync for SharedMemoryManager {}
