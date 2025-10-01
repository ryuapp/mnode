use crate::add_internal_function;
use rquickjs::Ctx;
use serde_json::json;
use std::fs;

#[cfg(windows)]
fn get_windows_file_info(path: &str) -> std::result::Result<(u64, u64, u32, u64), String> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::Storage::FileSystem::{
        BY_HANDLE_FILE_INFORMATION, CreateFileW, FILE_FLAG_BACKUP_SEMANTICS, FILE_GENERIC_READ,
        FILE_SHARE_READ, GetFileInformationByHandle, OPEN_EXISTING,
    };

    let wide_path: Vec<u16> = OsStr::new(path)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    unsafe {
        let handle = CreateFileW(
            windows::core::PCWSTR(wide_path.as_ptr()),
            FILE_GENERIC_READ.0,
            FILE_SHARE_READ,
            None,
            OPEN_EXISTING,
            FILE_FLAG_BACKUP_SEMANTICS,
            None,
        )
        .map_err(|e| format!("Failed to open file: {}", e))?;

        let mut file_info: BY_HANDLE_FILE_INFORMATION = std::mem::zeroed();
        GetFileInformationByHandle(handle, &mut file_info)
            .map_err(|e| format!("Failed to get file info: {}", e))?;

        let _ = CloseHandle(handle);

        let dev = file_info.dwVolumeSerialNumber as u64;
        let ino = ((file_info.nFileIndexHigh as u64) << 32) | (file_info.nFileIndexLow as u64);
        let nlink = file_info.nNumberOfLinks;

        // For now, use file size for allocation. In a real implementation,
        // we'd need NtQueryInformationFile to get AllocationSize.
        // Small files often have AllocationSize = 0 on Windows.
        let file_size = ((file_info.nFileSizeHigh as u64) << 32) | (file_info.nFileSizeLow as u64);
        let allocation_size = if file_size == 0 { 0 } else { file_size };

        Ok((dev, ino, nlink, allocation_size))
    }
}

#[cfg(windows)]
fn get_blksize(path: &str) -> std::result::Result<u64, String> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use windows::Win32::Storage::FileSystem::GetDiskFreeSpaceW;

    let abs_path = std::path::Path::new(path)
        .canonicalize()
        .map_err(|e| format!("Failed to canonicalize path: {}", e))?;

    let root = abs_path
        .ancestors()
        .last()
        .ok_or_else(|| "Path has no root".to_string())?;

    let wide_root: Vec<u16> = OsStr::new(root)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    unsafe {
        let mut sectors_per_cluster = 0u32;
        let mut bytes_per_sector = 0u32;
        let mut _number_of_free_clusters = 0u32;
        let mut _total_number_of_clusters = 0u32;

        GetDiskFreeSpaceW(
            windows::core::PCWSTR(wide_root.as_ptr()),
            Some(&mut sectors_per_cluster),
            Some(&mut bytes_per_sector),
            Some(&mut _number_of_free_clusters),
            Some(&mut _total_number_of_clusters),
        )
        .map_err(|e| format!("Failed to get disk free space: {}", e))?;

        Ok((bytes_per_sector * sectors_per_cluster) as u64)
    }
}

pub fn setup(ctx: &Ctx) -> std::result::Result<(), Box<dyn std::error::Error>> {
    add_internal_function!(ctx, "readFileSync", |path: String| {
        read_file_sync(path)
            .unwrap_or_else(|e| format!(r#"{{"error":"{}"}}"#, e.replace('"', "\\\"")))
    });

    add_internal_function!(ctx, "writeFileSync", |path: String, data: String| {
        write_file_sync(path, data).map(|_| 0).unwrap_or_else(|e| {
            eprintln!("{}", e);
            -1
        })
    });

    add_internal_function!(ctx, "existsSync", |path: String| {
        exists_sync(path).unwrap_or(false)
    });

    add_internal_function!(ctx, "statSync", |path: String| {
        stat_sync(path).unwrap_or_else(|e| format!(r#"{{"error":"{}"}}"#, e.replace('"', "\\\"")))
    });

    add_internal_function!(ctx, "readdirSync", |path: String| {
        readdir_sync(path)
            .unwrap_or_else(|e| format!(r#"{{"error":"{}"}}"#, e.replace('"', "\\\"")))
    });

    Ok(())
}

pub fn read_file_sync(path: String) -> std::result::Result<String, String> {
    fs::read_to_string(&path).map_err(|e| format!("Failed to read file {}: {}", path, e))
}

pub fn write_file_sync(path: String, data: String) -> std::result::Result<i32, String> {
    fs::write(&path, data).map_err(|e| format!("Failed to write file {}: {}", path, e))?;
    Ok(0)
}

pub fn exists_sync(path: String) -> std::result::Result<bool, String> {
    Ok(fs::metadata(&path).is_ok())
}

pub fn stat_sync(path: String) -> std::result::Result<String, String> {
    let metadata =
        fs::metadata(&path).map_err(|e| format!("Failed to stat file {}: {}", path, e))?;

    // Convert times with higher precision (avoid as_secs_f64 precision loss)
    let mtime = {
        let duration = metadata
            .modified()
            .unwrap_or(std::time::UNIX_EPOCH)
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();
        duration.as_secs() as f64 * 1000.0
            + duration.subsec_millis() as f64
            + duration.subsec_nanos() as f64 % 1_000_000.0 / 1_000_000.0
    };

    let atime = {
        let duration = metadata
            .accessed()
            .unwrap_or(std::time::UNIX_EPOCH)
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();
        duration.as_secs() as f64 * 1000.0
            + duration.subsec_millis() as f64
            + duration.subsec_nanos() as f64 % 1_000_000.0 / 1_000_000.0
    };

    // ctime = change time (metadata modification time)
    // On Windows, use modified() as closest approximation to ChangeTime
    let ctime = {
        let duration = metadata
            .modified()
            .unwrap_or(std::time::UNIX_EPOCH)
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();
        duration.as_secs() as f64 * 1000.0
            + duration.subsec_millis() as f64
            + duration.subsec_nanos() as f64 % 1_000_000.0 / 1_000_000.0
    };

    // birthtime = creation time
    let birthtime = {
        let duration = metadata
            .created()
            .unwrap_or(std::time::UNIX_EPOCH)
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();
        duration.as_secs() as f64 * 1000.0
            + duration.subsec_millis() as f64
            + duration.subsec_nanos() as f64 % 1_000_000.0 / 1_000_000.0
    };

    // Get real filesystem metadata
    #[cfg(windows)]
    let (dev, ino, nlink, allocation_size) = get_windows_file_info(&path)?;

    #[cfg(windows)]
    let blksize = get_blksize(&path).unwrap_or(4096);

    #[cfg(windows)]
    let (uid, gid, rdev) = (0u32, 0u32, 0u64);

    #[cfg(not(windows))]
    let (dev, ino, nlink, uid, gid, rdev, blksize) = {
        use std::os::unix::fs::MetadataExt;
        (
            metadata.dev(),
            metadata.ino(),
            metadata.nlink() as u32,
            metadata.uid(),
            metadata.gid(),
            metadata.rdev(),
            metadata.blksize(),
        )
    };

    #[cfg(windows)]
    let mode = {
        const S_IFDIR: u32 = 0o040000;
        const S_IFREG: u32 = 0o100000;
        const S_IREAD: u32 = 0o000400;
        const S_IWRITE: u32 = 0o000200;

        let mut mode = 0u32;

        if metadata.is_dir() {
            mode |= S_IFDIR;
        } else {
            mode |= S_IFREG;
        }

        if metadata.permissions().readonly() {
            mode |= S_IREAD | (S_IREAD >> 3) | (S_IREAD >> 6);
        } else {
            mode |=
                (S_IREAD | S_IWRITE) | ((S_IREAD | S_IWRITE) >> 3) | ((S_IREAD | S_IWRITE) >> 6);
        }

        mode
    };

    #[cfg(not(windows))]
    let mode = {
        use std::os::unix::fs::PermissionsExt;
        metadata.permissions().mode()
    };

    // Calculate blocks (512-byte units like libuv)
    #[cfg(windows)]
    let blocks = allocation_size / 512;

    #[cfg(not(windows))]
    let blocks = {
        use std::os::unix::fs::MetadataExt;
        metadata.blocks()
    };

    let stat = json!({
        "dev": dev,
        "ino": ino,
        "mode": mode,
        "nlink": nlink,
        "uid": uid,
        "gid": gid,
        "rdev": rdev,
        "size": metadata.len(),
        "blksize": blksize,
        "blocks": blocks,
        "atimeMs": atime,
        "mtimeMs": mtime,
        "ctimeMs": ctime,
        "birthtimeMs": birthtime,
        "isFile": metadata.is_file(),
        "isDirectory": metadata.is_dir(),
        "isSymbolicLink": false,
        "isBlockDevice": false,
        "isCharacterDevice": false,
        "isFIFO": false,
        "isSocket": false,
    });

    Ok(serde_json::to_string(&stat).unwrap())
}

pub fn readdir_sync(path: String) -> std::result::Result<String, String> {
    let entries = fs::read_dir(&path)
        .map_err(|e| format!("Failed to read directory {}: {}", path, e))?
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| entry.file_name().into_string().ok())
        .collect::<Vec<String>>();

    Ok(serde_json::to_string(&entries).unwrap())
}
