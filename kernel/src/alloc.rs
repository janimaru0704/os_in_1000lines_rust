use crate::{linker, my_panic};

pub const PAGE_SIZE: usize = 4096;

pub const SATP_SV32: u32 = 1 << 31;

pub const PAGE_V: u32 = 1 << 0;
pub const PAGE_R: u32 = 1 << 1;
pub const PAGE_W: u32 = 1 << 2;
pub const PAGE_X: u32 = 1 << 3;
pub const PAGE_U: u32 = 1 << 4;

pub type PAddr = u32;

pub fn alloc_pages(n: usize) -> PAddr {
    static mut NEXT_PADDR: PAddr = 0;

    unsafe {
        if NEXT_PADDR == 0 {
            NEXT_PADDR = &raw const linker::__free_ram as PAddr;
        }
        let paddr = NEXT_PADDR;

        let size = n * PAGE_SIZE;
        NEXT_PADDR += size as u32;

        if NEXT_PADDR > &raw const linker::__free_ram_end as PAddr {
            my_panic!("out of memory");
        }

        common::memset(paddr as *mut u8, 0, size);
        paddr
    }
}

pub unsafe fn map_page(table1: *mut u32, vaddr: usize, paddr: PAddr, flags: u32) {
    if !common::is_aligned(vaddr, PAGE_SIZE) {
        my_panic!("unaligned vaddr {}", vaddr);
    }

    if !common::is_aligned(paddr as usize, PAGE_SIZE) {
        my_panic!("unaligned paddr {}", paddr);
    }

    unsafe {
        let vpn1 = (vaddr >> 22) & 0x3ff;
        let entry1 = table1.add(vpn1);
        if (*entry1 & PAGE_V) == 0 {
            let pt_paddr = alloc_pages(1);
            *entry1 = ((pt_paddr / PAGE_SIZE as u32) << 10) | PAGE_V;
        }

        let vpn0 = (vaddr >> 12) & 0x3ff;
        let table0 = ((*entry1 >> 10) as usize * PAGE_SIZE) as *mut u32;
        let entry0 = table0.add(vpn0);
        *entry0 = ((paddr / PAGE_SIZE as u32) << 10) | flags | PAGE_V;
    }
}
