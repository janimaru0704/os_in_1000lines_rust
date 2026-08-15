use crate::{my_panic, println, virtio};

pub const FILES_MAX: usize = 2;
pub const DISK_MAX_SIZE: usize = common::align_up(
    core::mem::size_of::<File>() * FILES_MAX, virtio::SECTOR_SIZE
);

#[repr(C, packed)]
pub struct TarHeader {
    pub name: [u8; 100],
    pub mode: [u8; 8],
    pub uid: [u8; 8],
    pub gid: [u8; 8],
    pub size: [u8; 12],
    pub mtime: [u8; 12],
    pub checksum: [u8; 8],
    pub type_: u8,
    pub linkname: [u8; 100],
    pub magic: [u8; 6],
    pub version: [u8; 2],
    pub uname: [u8; 32],
    pub gname: [u8; 32],
    pub devmajor: [u8; 8],
    pub devminor: [u8; 8],
    pub prefix: [u8; 155],
    pub padding: [u8; 12],
}

#[repr(C)]
pub struct File {
    pub in_use: bool,
    pub name: [u8; 100],
    pub data: [u8; 1024],
    pub size: usize,
}

impl File {
    const fn new() -> Self {
        Self {
            in_use: false,
            name: [0; 100],
            data: [0; 1024],
            size: 0,
        }
    }
}

pub static mut FILES: [File; FILES_MAX] = [const { File::new() }; FILES_MAX];
pub static mut DISK: [u8; DISK_MAX_SIZE] = [0; DISK_MAX_SIZE];

unsafe fn oct2int(oct: &[u8]) -> i32 {
    let mut dec = 0;

    for &ch in oct {
        if ch < b'0' || ch > b'7' {
            break;
        }

        dec = dec * 8 + (ch - b'0') as i32;
    }

    dec
}

pub fn fs_init() {
    for sector in 0..(core::mem::size_of::<u8>() * DISK_MAX_SIZE / virtio::SECTOR_SIZE) {
        unsafe{
            virtio::read_write_disk(&raw mut DISK[sector * virtio::SECTOR_SIZE], sector as u64, false);
        }
    }

    let mut off = 0;
    for i in 0..FILES_MAX {
        unsafe {
            let header = &*(&raw const DISK[off] as *const TarHeader);
            if header.name[0] == b'\0' {
                break;
            }

            if common::strcmp(header.magic.as_ptr(), b"ustar".as_ptr()) != 0 {
                my_panic!(
                    "invalid tar header: magic=\"{}\"",
                    core::str::from_utf8_unchecked(
                        &header.magic[..header.magic.iter()
                            .position(|&x| x == 0)
                            .unwrap_or(6)],
                    ),
                );
            }

            let filesz = oct2int(&header.size);
            let file = &mut FILES[i];
            file.in_use = true;
            common::strcpy(file.name.as_mut_ptr(), header.name.as_ptr());

            let data = &raw const DISK[off + core::mem::size_of::<TarHeader>()];

            common::memcpy(file.data.as_mut_ptr(), data, filesz as usize);
            file.size = filesz as usize;
            println!(
                "file: {}, size={}",
                core::str::from_utf8_unchecked(
                    &file.name[..file.name.iter()
                    .position(|&x| x == 0)
                    .unwrap_or(100)]
                ),
                file.size
            );

            off += common::align_up(core::mem::size_of::<TarHeader>() + filesz as usize, virtio::SECTOR_SIZE);
        }
    }
}