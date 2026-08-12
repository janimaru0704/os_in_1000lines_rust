use crate::{linker, panic};

pub const PAGE_SIZE: usize = 4096;

pub const SATP_SV32: usize = 1 << 31;

pub const PAGE_V: usize = 1 << 0;
pub const PAGE_R: usize = 1 << 1;
pub const PAGE_W: usize = 1 << 2;
pub const PAGE_X: usize = 1 << 3;
pub const PAGE_U: usize = 1 << 4;

pub type PAddr = usize;

pub fn alloc_pages(n: usize) -> PAddr {
    static mut NEXT_PADDR: PAddr = 0;

    unsafe {
        if NEXT_PADDR == 0 {
            NEXT_PADDR = &raw const linker::__free_ram as PAddr;
        }
        let paddr = NEXT_PADDR;
        NEXT_PADDR += n * PAGE_SIZE;

        if NEXT_PADDR > &raw const linker::__free_ram_end as PAddr {
            panic!("out of memory");
        }

        common::memset(paddr as *mut u8, 0, n * PAGE_SIZE);
        paddr
    }
}

pub unsafe fn map_page(table1: *mut usize, vaddr: usize, paddr: PAddr, flags: usize) {
    if !common::is_aligned(vaddr, PAGE_SIZE) {
        panic!("unaligned vaddr {}", vaddr);
    }

    if !common::is_aligned(paddr, PAGE_SIZE) {
        panic!("unaligned paddr {}", paddr);
    }

    unsafe {
        let vpn1 = (vaddr >> 22) & 0x3ff;
        let entry1 = table1.add(vpn1);
        if (*entry1 & PAGE_V) == 0 {
            let pt_paddr = alloc_pages(1);
            *entry1 = ((pt_paddr / PAGE_SIZE) << 10) | PAGE_V;
        }

        let vpn0 = (vaddr >> 12) & 0x3ff;
        let table0 = ((*entry1 >> 10) * PAGE_SIZE) as *mut usize;
        let entry0 = table0.add(vpn0);
        *entry0 = ((paddr / PAGE_SIZE) << 10) | flags | PAGE_V;
    }
}
