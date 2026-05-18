#!/bin/bash
set -e

if [ ! -d ./esp/efi/boot ]; then
	echo "You do not have a /esp/efi/boot folder set up here! Take a gander at: https://rust-osdev.github.io/uefi-rs/tutorial/vm.html#firmware-files"
	exit 1
fi

cargo build --target x86_64-unknown-uefi --release -p reimuefi

cp target/x86_64-unknown-uefi/release/reimuefi.efi esp/efi/boot/bootx64.efi

qemu-system-x86_64 -enable-kvm \
	-drive if=pflash,format=raw,readonly=on,file=OVMF_CODE.4m.fd \
	-drive if=pflash,format=raw,readonly=on,file=OVMF_VARS.4m.fd \
	-drive format=raw,file=fat:rw:esp
