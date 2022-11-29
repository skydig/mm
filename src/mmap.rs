
// Note: this requires the `cargo` feature
use std::ffi::{CString,c_void};


pub fn ioremap32_read(addr:u32)->Result<u32,String> {
    let mask = (4096-1) as u32;
    let ofst= addr as u32 & mask;
    let page = addr as u32 & !mask;
    let len=4;

    unsafe {
        let cs = CString::new("/dev/mem").unwrap();
        let fd = libc::open(cs.as_ptr(),libc::O_SYNC|libc::O_RDWR);
        if fd < 0 {
            Err("open /dev/mem error")?
        }
        #[cfg(any(target_arch = "aarch64", target_arch="x86_64"))]
        let vaddr = libc::mmap(0 as *mut c_void, (ofst+len) as usize,libc::PROT_READ|libc::PROT_WRITE,libc::MAP_SHARED, fd, page as i64);
        #[cfg(target_arch = "powerpc")]
        let vaddr = libc::mmap(0 as *mut c_void, (ofst+len) as usize,libc::PROT_READ|libc::PROT_WRITE,libc::MAP_SHARED, fd, page as i32);
        let mapped = std::slice::from_raw_parts_mut(vaddr as *mut u8, len as usize+ofst as usize);
        let mut t_vec=vec![0u8;len as usize+ofst as usize];

        for i in 0..t_vec.len() {
            t_vec[i]=mapped[i];
        }
        let _4_array=[t_vec[ofst as usize], t_vec[ofst as usize+1], t_vec[ofst as usize+2], t_vec[ofst as usize+3]];
        let ret = u32::from_be_bytes(_4_array);
        libc::munmap(vaddr,(ofst+len) as usize);
        libc::close(fd);
        return Ok(ret);
    }
}
#[test] fn basic_test() {
    ioremap(0x1004);
}

pub fn ioremap(addr:u32)->Result<*mut c_void,String> {
    let mask = (4096-1) as u32;
    let ofst= addr as u32 & mask;
    let page = addr as u32 & !mask;
    let len=4;

    unsafe {
        let cs = CString::new("/dev/mem").unwrap();
        let fd = libc::open(cs.as_ptr(),libc::O_SYNC|libc::O_RDWR);
        if fd < 0 {
            Err("open /dev/mem error")?
        }
        #[cfg(any(target_arch = "aarch64", target_arch="x86_64"))]
        let mut vaddr = libc::mmap(0 as *mut c_void, (ofst+len) as usize,libc::PROT_READ|libc::PROT_WRITE,libc::MAP_SHARED, fd, page as i64);
        #[cfg(target_arch = "powerpc")]
        let vaddr = libc::mmap(0 as *mut c_void, (ofst+len) as usize,libc::PROT_READ|libc::PROT_WRITE,libc::MAP_SHARED, fd, page as i32);
        vaddr = vaddr.add(ofst as usize);
        return Ok(vaddr);
    }
}
