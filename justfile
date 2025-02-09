build:
    cargo build --release
    cp target/x86_64-unknown-none/release/piuma kernel

configure-limine:
    #!/usr/bin/env bash
    set -euxo pipefail
    cd limine

    ./bootstrap
    ./configure --enable-uefi-x86-64

build-limine:
    cd limine && make

create-disk:
    #!/usr/bin/guestfish -f
    echo "Creating image"
    disk-create disk.qcow2 qcow2 10G
    add disk.qcow2
    run

    echo "Partitioning image"
    part_init /dev/sda gpt
    part_add /dev/sda primary 2048 2099199
    part_add /dev/sda primary 2099200 -2048
    part-set-gpt-type /dev/sda 1 C12A7328-F81F-11D2-BA4B-00A0C93EC93B 

    echo "Formatting partitions"
    mkfs vfat /dev/sda1

    echo "Mounting efi partition"
    mount /dev/sda1 /
    mkdir /boot
    mkdir-p /EFI/BOOT

    echo "Copying to efi partition"
    copy-in kernel /boot
    copy-in limine/bin/BOOTX64.EFI /EFI/BOOT
    copy-in limine.conf /boot

    echo "Unmounting efi partition"
    umount /
