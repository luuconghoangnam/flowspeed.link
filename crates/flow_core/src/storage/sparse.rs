use std::fs::File;
use std::io;

#[cfg(windows)]
use std::os::windows::io::AsRawHandle;

/// Thiết lập thuộc tính Sparse File trên hệ điều hành Windows NTFS
/// Ánh xạ 1:1 từ com.flowspeed.lib.downloader.download.SparseFile
pub fn mark_as_sparse_file(_file: &File) -> io::Result<()> {
    #[cfg(windows)]
    {
        use windows_sys::Win32::Foundation::HANDLE;
        use windows_sys::Win32::System::Ioctl::FSCTL_SET_SPARSE;
        use windows_sys::Win32::System::IO::DeviceIoControl;

        let handle = _file.as_raw_handle() as HANDLE;
        let mut bytes_returned = 0u32;

        let success = unsafe {
            DeviceIoControl(
                handle,
                FSCTL_SET_SPARSE,
                std::ptr::null_mut(),
                0,
                std::ptr::null_mut(),
                0,
                &mut bytes_returned,
                std::ptr::null_mut(),
            )
        };

        if success == 0 {
            return Err(io::Error::last_os_error());
        }
    }

    Ok(())
}
