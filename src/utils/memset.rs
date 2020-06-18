pub fn memset(slice: &mut [u8], value: u8) {
    unsafe {
        std::ptr::write_bytes(slice.as_mut_ptr(), value, slice.len());
    }
}
