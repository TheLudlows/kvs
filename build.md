curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

yum install openssl openssl-devel
yum -y install gcc
yum -y install gcc-c++
yum -y install cmake
yum -y install zip

git clone https://github.com/TheLudlows/kvs.git


1828723137315221