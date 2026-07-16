# HANDWRITE-BEGIN gap="missing-generator:ephemeral-k8s-machine-image" tracker="#1693" reason="The embedded K3s machine substrate is a host-runtime build asset, not a generated Rust source unit."
# This asset is embedded in the released VAT binary. Apple Container machines
# boot the image init process, so an ordinary application image is not a usable
# K3s substrate. This is deliberately the Phase-0-proven Ubuntu/systemd setup;
# pinning and package minimization are follow-up hardening, not an implicit
# claim of reproducibility.
FROM ubuntu:24.04

ENV container=container

RUN apt-get update && \
    apt-get install -y \
        dbus \
        systemd \
        openssh-server \
        net-tools \
        iproute2 \
        iputils-ping \
        curl \
        wget \
        vim-tiny \
        man \
        sudo && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/* && \
    yes | unminimize

RUN : > /etc/machine-id && \
    : > /var/lib/dbus/machine-id && \
    systemctl set-default multi-user.target && \
    systemctl mask \
        dev-hugepages.mount \
        sys-fs-fuse-connections.mount \
        systemd-update-utmp.service \
        systemd-tmpfiles-setup.service \
        console-getty.service && \
    systemctl disable networkd-dispatcher.service && \
    sed -i -e 's/^AcceptEnv LANG LC_\\*$/#AcceptEnv LANG LC_*/' /etc/ssh/sshd_config
# HANDWRITE-END
