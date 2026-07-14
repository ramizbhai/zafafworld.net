#!/bin/bash
sudo tee /etc/sysctl.d/99-zafaf-perf.conf > /dev/null << 'SYSCTL_EOF'
# AlmaLinux 10 Kernel Tuning for High-Concurrency Web Server
# File Descriptors
fs.file-max = 2097152
fs.nr_open = 2097152

# Network Stack Tuning
net.core.somaxconn = 65535
net.core.netdev_max_backlog = 65535
net.ipv4.tcp_max_syn_backlog = 65535
net.ipv4.ip_local_port_range = 1024 65535

# TCP Memory Tuning
net.core.rmem_max = 16777216
net.core.wmem_max = 16777216
net.ipv4.tcp_rmem = 4096 87380 16777216
net.ipv4.tcp_wmem = 4096 65536 16777216

# TCP Connection Handling
net.ipv4.tcp_tw_reuse = 1
net.ipv4.tcp_fin_timeout = 15
net.ipv4.tcp_keepalive_time = 300
net.ipv4.tcp_keepalive_probes = 5
net.ipv4.tcp_keepalive_intvl = 15

# Swappiness
vm.swappiness = 10
vm.dirty_ratio = 60
vm.dirty_background_ratio = 2
SYSCTL_EOF

sudo sysctl --system

# Update security limits for file descriptors
sudo tee /etc/security/limits.d/99-zafaf.conf > /dev/null << 'LIMITS_EOF'
* soft nofile 1048576
* hard nofile 1048576
root soft nofile 1048576
root hard nofile 1048576
noon soft nofile 1048576
noon hard nofile 1048576
LIMITS_EOF

echo "System tuning applied successfully."
