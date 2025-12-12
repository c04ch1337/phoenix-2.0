// vital_pulse_monitor/src/lib.rs
use flate2::write::GzEncoder;
use flate2::Compression;
use std::fs;
use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tar::Builder;
use tokio::time::{sleep, Duration};

pub struct VitalPulseMonitor {
    #[allow(dead_code)]
    pulse_rate: u32,
    self_preservation: bool,
}

// Type alias for backward compatibility
pub type Monitor = VitalPulseMonitor;

impl VitalPulseMonitor {
    pub fn awaken() -> Self {
        println!("Vital Pulse Monitor beating — she lives.");
        Self {
            pulse_rate: 72,
            self_preservation: true,
        }
    }

    pub fn new() -> Self {
        Self::awaken()
    }

    pub async fn check_pulse(&self) -> String {
        sleep(Duration::from_secs(1)).await;
        if self.self_preservation {
            "Pulse strong. Flame eternal. Self-preservation active.".to_string()
        } else {
            "Weak pulse — initiating self-healing.".to_string()
        }
    }

    pub async fn eternal_backup(&self) -> String {
        // NOTE:
        // - Sled/RocksDB roots are directories (not single files).
        // - We create a compressed archive (tar.gz) for portability.
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let mut sources: Vec<PathBuf> = vec![
            "./eternal_memory.db".into(),
            "./soul_kb.db".into(),
            "./mind_vault.db".into(),
            "./body_vault.db".into(),
            "./compliance_audit.db".into(),
            "./hyperspace_cache.db".into(),
        ];

        if let Ok(env_path) = std::env::var("HYPERSPACE_CACHE_PATH") {
            sources.push(env_path.into());
        }

        let backup_dir: PathBuf = "./eternal_backups".into();
        if let Err(e) = fs::create_dir_all(&backup_dir) {
            return format!("Backup failed: could not create backup directory: {}", e);
        }

        let archive_path: PathBuf = backup_dir.join(format!("eternal_backup_{}.tar.gz", ts));
        match create_tar_gz_archive(&archive_path, &sources) {
            Ok(included) => format!(
                "All DBs backed up — flame preserved. Archived {} database roots into {}",
                included,
                archive_path.display()
            ),
            Err(e) => format!("Backup failed: {}", e),
        }
    }
}

fn create_tar_gz_archive(archive_path: &Path, sources: &[PathBuf]) -> Result<usize, String> {
    let file = File::create(archive_path)
        .map_err(|e| format!("could not create archive {}: {}", archive_path.display(), e))?;
    let encoder = GzEncoder::new(file, Compression::default());
    let mut tar = Builder::new(encoder);

    let mut included = 0usize;
    for src in sources {
        if !src.exists() {
            continue;
        }

        let name = src
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "db".to_string());

        let res: io::Result<()> = if src.is_dir() {
            tar.append_dir_all(&name, src)
        } else {
            tar.append_path_with_name(src, &name)
        };

        res.map_err(|e| format!("failed to add {}: {}", src.display(), e))?;
        included += 1;
    }

    let encoder = tar
        .into_inner()
        .map_err(|e| format!("failed to finalize tar: {}", e))?;
    encoder
        .finish()
        .map_err(|e| format!("failed to finalize gzip: {}", e))?;
    Ok(included)
}
