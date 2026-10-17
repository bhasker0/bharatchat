use std::ffi::CString;
use std::os::raw::{c_char, c_int};
use std::slice;
use sovereign_core::{IdentityKeyPair, compute_canonical_cid, compute_perceptual_hash};

/// Generates a new hardware-rooted W3C DID, writing the null-terminated string to `out_buffer`.
/// Returns 0 on success, -1 on buffer overflow.
#[no_mangle]
pub unsafe extern "C" fn sovereign_generate_did(
    out_buffer: *mut c_char,
    max_len: usize,
) -> c_int {
    if out_buffer.is_null() || max_len == 0 {
        return -1;
    }

    let identity = IdentityKeyPair::generate();
    let c_str = match CString::new(identity.did.clone()) {
        Ok(s) => s,
        Err(_) => return -1,
    };

    let bytes = c_str.as_bytes_with_nul();
    if bytes.len() > max_len {
        return -1;
    }

    std::ptr::copy_nonoverlapping(bytes.as_ptr() as *const c_char, out_buffer, bytes.len());
    0
}

/// Computes the 32-byte BLAKE3 canonical content hash into `out_cid_32`
#[no_mangle]
pub unsafe extern "C" fn sovereign_compute_cid(
    content_ptr: *const u8,
    content_len: usize,
    out_cid_32: *mut u8,
) -> c_int {
    if content_ptr.is_null() || out_cid_32.is_null() {
        return -1;
    }

    let content_slice = slice::from_raw_parts(content_ptr, content_len);
    let cid = compute_canonical_cid(content_slice);

    std::ptr::copy_nonoverlapping(cid.as_ptr(), out_cid_32, 32);
    0
}

/// Computes the 64-bit perceptual hash (dHash) for client-side CSAM checking
#[no_mangle]
pub unsafe extern "C" fn sovereign_compute_phash(
    media_ptr: *const u8,
    media_len: usize,
    out_phash: *mut u64,
) -> c_int {
    if media_ptr.is_null() || out_phash.is_null() {
        return -1;
    }

    let media_slice = slice::from_raw_parts(media_ptr, media_len);
    let phash = compute_perceptual_hash(media_slice);

    *out_phash = phash;
    0
}

/// Helper to free C-allocated strings
#[no_mangle]
pub unsafe extern "C" fn sovereign_free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        let _ = CString::from_raw(ptr);
    }
}
