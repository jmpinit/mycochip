use simavr_ffi as ffi;
use std::{alloc, ffi::CString, path::Path, ptr::NonNull};
use super::avr::Avr;

pub struct Firmware {
    ptr: NonNull<ffi::elf_firmware_t>,
}

impl Firmware {
    pub fn new() -> Self {
        // Safety: We know that `elf_firmware_t`'s layout has a non-zero size.
        //
        // (we also use `alloc_zeroed`, because that's what simavr's docs
        // suggest to do.)
        let ptr = unsafe {
            alloc::alloc_zeroed(alloc::Layout::new::<ffi::elf_firmware_t>())
                as *mut ffi::elf_firmware_t
        };

        // Unwrap-safety: This can fail only if the underlying allocator failed
        // to find enough memory to allocate the chunk - in that case panicking
        // is the best we can afford anyway
        let ptr = NonNull::new(ptr).unwrap();

        Self { ptr }
    }

    pub fn load_elf(self, path: impl AsRef<Path>) -> Self {
        let path = path.as_ref().display().to_string();

        // Unwrap-safety: Paths cannot contain null-terminators, so a string
        // we've got from `.display().to_string()` cannot either
        let c_path = CString::new(path).unwrap();

        // Safety: `self.ptr` points at a valid, zeroed instance of
        // `elf_firmware_t`; `c_path` points at a valid `CString`
        let status = unsafe { ffi::elf_read_firmware(c_path.as_ptr(), self.ptr.as_ptr()) };

        if status != 0 {
            panic!(
                "Couldn't load firmware from: {} (status = {})",
                c_path.into_string().unwrap(),
                status
            );
        }

        self
    }

    pub fn write_eeprom(self, data: &[u8]) -> Firmware {
        const EESIZE: usize = 1024;

        if data.len() > EESIZE {
            panic!("EEPROM data too large ({} > {})", data.len(), EESIZE);
        }

        unsafe {
            let fw = &mut *self.ptr.as_ptr();

            // Allocate 1024 bytes for the eeprom
            let layout = alloc::Layout::from_size_align(1024, std::mem::align_of::<u8>()).unwrap();
            fw.eeprom = alloc::alloc_zeroed(layout);
            fw.eesize = EESIZE as u32;

            // Copy the data to the eeprom
            std::ptr::copy_nonoverlapping(data.as_ptr(), fw.eeprom, data.len());
        }

        self
    }

    pub fn flash_to(self, avr: &mut Avr) {
        avr.load_firmware(self.ptr);
    }
}

impl Drop for Firmware {
    fn drop(&mut self) {
        unsafe {
            drop(Box::from_raw(self.ptr.as_ptr()));
        }
    }
}
