use crate::{alloc, my_panic, println};

pub const SECTOR_SIZE: usize = 512;
pub const VIRTQ_ENTRY_NUM: usize = 16;

pub const VIRTIO_DEVICE_BLK: u32 = 2;
pub const VIRTIO_BLK_PADDR: u32 = 0x10001000;

pub const VIRTIO_REG_MAGIC: usize = 0x00;
pub const VIRTIO_REG_VERSION: usize = 0x04;
pub const VIRTIO_REG_DEVICE_ID: usize = 0x08;
pub const VIRTIO_REG_PAGE_SIZE: usize = 0x28;
pub const VIRTIO_REG_QUEUE_SEL: usize = 0x30;
pub const VIRTIO_REG_QUEUE_NUM_MAX: usize = 0x34;
pub const VIRTIO_REG_QUEUE_NUM: usize = 0x38;
pub const VIRTIO_REG_QUEUE_PFN: usize = 0x40;
pub const VIRTIO_REG_QUEUE_READY: usize = 0x44;
pub const VIRTIO_REG_QUEUE_NOTIFY: usize = 0x50;
pub const VIRTIO_REG_DEVICE_STATUS: usize = 0x70;
pub const VIRTIO_REG_DEVICE_CONFIG: usize = 0x100;

pub const VIRTIO_STATUS_ACK: u32 = 1;
pub const VIRTIO_STATUS_DRIVER: u32 = 2;
pub const VIRTIO_STATUS_DRIVER_OK: u32 = 4;

pub const VIRTQ_DESC_F_NEXT: u32 = 1;
pub const VIRTQ_DESC_F_WRITE: u32 = 2;

pub const VIRTQ_AVAIL_F_NO_INTERRUPT: u32 = 1;

pub const VIRTIO_BLK_T_IN: u32 = 0;
pub const VIRTIO_BLK_T_OUT: u32 = 1;

#[repr(C, packed)]
pub struct VirtqDesc {
    pub addr: u64,
    pub len: u32,
    pub flags: u16,
    pub next: u16,
}

#[repr(C, packed)]
pub struct VirtqAvail {
    pub flags: u16,
    pub index: u16,
    pub ring: [u16; VIRTQ_ENTRY_NUM],
}

#[repr(C, packed)]
pub struct VirtqUsedElem {
    pub id: u32,
    pub len: u32,
}

#[repr(C, packed)]
pub struct VirtqUsed {
    pub flags: u16,
    pub index: u16,
    pub ring: [VirtqUsedElem; VIRTQ_ENTRY_NUM],
}

const DESCS_AVAIL_SIZE: usize =
    core::mem::size_of::<[VirtqDesc; VIRTQ_ENTRY_NUM]>() + core::mem::size_of::<VirtqAvail>();

const PADDING_SIZE: usize = common::align_up(DESCS_AVAIL_SIZE, alloc::PAGE_SIZE) - DESCS_AVAIL_SIZE;

#[repr(C, packed)]
pub struct VirtioVirtq {
    pub descs: [VirtqDesc; VIRTQ_ENTRY_NUM],
    pub avail: VirtqAvail,
    _padding: [u8; PADDING_SIZE],
    pub used: VirtqUsed,
    pub queue_index: i32,
    pub used_index: *mut u16,
    pub last_used_index: u16,
}

#[repr(C, packed)]
pub struct VirtioBlkReq {
    pub type_: u32,
    pub reserved: u32,
    pub sector: u64,
    pub data: [u8; 512],
    pub status: u8,
}

unsafe fn virtio_reg_read32(offset: usize) -> u32 {
    unsafe { core::ptr::read_volatile((VIRTIO_BLK_PADDR as usize + offset) as *const u32) }
}

unsafe fn virtio_reg_read64(offset: usize) -> u64 {
    unsafe { core::ptr::read_volatile((VIRTIO_BLK_PADDR as usize + offset) as *const u64) }
}

unsafe fn virtio_reg_write32(offset: usize, value: u32) {
    unsafe {
        core::ptr::write_volatile((VIRTIO_BLK_PADDR as usize + offset) as *mut u32, value);
    }
}

unsafe fn virtio_reg_fetch_and_or32(offset: usize, value: u32) {
    unsafe {
        virtio_reg_write32(offset, virtio_reg_read32(offset) | value);
    }
}

static mut BLK_REQUEST_VQ: *mut VirtioVirtq = core::ptr::null_mut();
static mut BLK_REQ: *mut VirtioBlkReq = core::ptr::null_mut();
static mut BLK_REQ_PADDR: alloc::PAddr = 0;
static mut BLK_CAPACITY: u64 = 0;

unsafe fn virtq_init(index: usize) -> *mut VirtioVirtq {
    let virtq_paddr = alloc::alloc_pages(
        common::align_up(core::mem::size_of::<VirtioVirtq>(), alloc::PAGE_SIZE) / alloc::PAGE_SIZE,
    );
    let vq = virtq_paddr as *mut VirtioVirtq;
    unsafe {
        (*vq).queue_index = index as i32;
        (*vq).used_index = &raw mut (*vq).used.index;

        virtio_reg_write32(VIRTIO_REG_QUEUE_SEL, index as u32);
        virtio_reg_write32(VIRTIO_REG_QUEUE_NUM, VIRTQ_ENTRY_NUM as u32);
        virtio_reg_write32(VIRTIO_REG_QUEUE_PFN, virtq_paddr / alloc::PAGE_SIZE as u32);
    }
    vq
}

pub fn virtio_blk_init() {
    unsafe {
        if virtio_reg_read32(VIRTIO_REG_MAGIC) != 0x74726976 {
            my_panic!("virtio: invalid magic value");
        }
        if virtio_reg_read32(VIRTIO_REG_VERSION) != 1 {
            my_panic!("virtio: invalid version");
        }
        if virtio_reg_read32(VIRTIO_REG_DEVICE_ID) != VIRTIO_DEVICE_BLK {
            my_panic!("virtio: invalid device id");
        }

        virtio_reg_write32(VIRTIO_REG_DEVICE_STATUS, 0);
        virtio_reg_fetch_and_or32(VIRTIO_REG_DEVICE_STATUS, VIRTIO_STATUS_ACK);
        virtio_reg_fetch_and_or32(VIRTIO_REG_DEVICE_STATUS, VIRTIO_STATUS_DRIVER);
        virtio_reg_write32(VIRTIO_REG_PAGE_SIZE, alloc::PAGE_SIZE as u32);
        BLK_REQUEST_VQ = virtq_init(0);
        virtio_reg_write32(VIRTIO_REG_DEVICE_STATUS, VIRTIO_STATUS_DRIVER_OK);

        let capacity = virtio_reg_read64(VIRTIO_REG_DEVICE_CONFIG + 0) * SECTOR_SIZE as u64;
        BLK_CAPACITY = capacity;
        println!("virtio-blk: capacity is {} bytes", capacity);

        BLK_REQ_PADDR = alloc::alloc_pages(
            common::align_up(core::mem::size_of::<VirtioBlkReq>(), alloc::PAGE_SIZE)
                / alloc::PAGE_SIZE,
        );
        BLK_REQ = BLK_REQ_PADDR as *mut VirtioBlkReq;
    }
}

unsafe fn virtq_kick(vq: *mut VirtioVirtq, desc_index: i32) {
    unsafe {
        (*vq).avail.ring[(*vq).avail.index as usize % VIRTQ_ENTRY_NUM] = desc_index as u16;
        (*vq).avail.index += 1;
    }
    core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);
    unsafe {
        virtio_reg_write32(VIRTIO_REG_QUEUE_NOTIFY, (*vq).queue_index as u32);
        (*vq).last_used_index += 1;
    }
}

unsafe fn virtq_is_busy(vq: *mut VirtioVirtq) -> bool {
    unsafe { (*vq).last_used_index != *((*vq).used_index) }
}

pub unsafe fn read_write_disk(buf: *mut u8, sector: u64, is_write: bool) {
    let blk_capacity = unsafe { BLK_CAPACITY };
    if sector >= blk_capacity / SECTOR_SIZE as u64 {
        println!(
            "virtio: tried to read/write sector={}, but capacity is {}",
            sector,
            blk_capacity / SECTOR_SIZE as u64,
        );
        return;
    }

    let blk_req = unsafe { &mut *BLK_REQ };
    blk_req.sector = sector;
    blk_req.type_ = if is_write {
        VIRTIO_BLK_T_OUT
    } else {
        VIRTIO_BLK_T_IN
    };
    if is_write {
        unsafe {
            common::memcpy(blk_req.data.as_mut_ptr(), buf, SECTOR_SIZE);
        }
    }

    let vq = unsafe { BLK_REQUEST_VQ };
    let descs = unsafe { &mut (*vq).descs };

    let blk_req_paddr = unsafe { BLK_REQ_PADDR } as u64;

    descs[0].addr = blk_req_paddr;
    descs[0].len = (core::mem::size_of::<u32>() * 2 + core::mem::size_of::<u64>()) as u32;
    descs[0].flags = VIRTQ_DESC_F_NEXT as u16;
    descs[0].next = 1;

    descs[1].addr = blk_req_paddr + core::mem::offset_of!(VirtioBlkReq, data) as u64;
    descs[1].len = SECTOR_SIZE as u32;
    descs[1].flags = (VIRTQ_DESC_F_NEXT | if is_write { 0 } else { VIRTQ_DESC_F_WRITE }) as u16;
    descs[1].next = 2;

    descs[2].addr = blk_req_paddr + core::mem::offset_of!(VirtioBlkReq, status) as u64;
    descs[2].len = core::mem::size_of::<u8>() as u32;
    descs[2].flags = VIRTQ_DESC_F_WRITE as u16;

    unsafe {
        virtq_kick(vq, 0);

        while virtq_is_busy(vq) {}
    }

    if blk_req.status != 0 {
        println!(
            "virtio: warn: failed to read/write sector={} status={}",
            sector, blk_req.status
        );
        return;
    }

    if !is_write {
        unsafe {
            common::memcpy(buf, blk_req.data.as_ptr(), SECTOR_SIZE);
        }
    }
}
