# Installation from distributed .deb package
Download ngrokmonitor from github repo
`wget https://github.com/mixailom/ngrokmonitor/releases/download/0.0.2/ngrokmonitor.deb`

Install app and service:

`sudo dpkg -i ngrokmonitor.deb`

Modify ngrokmonitor.cfg config file and provide credentials, email address:

`sudo vi /etc/ngrokmonitor/ngrokmonitor.cfg`

Enable service, so it runs on startup: 

`sudo systemctl enable ngrokmonitor`

`sudo systemctl start ngrokmonitor`

# Building ngrokmonitor for ARM

## 1. Install OpenSSL Development Libraries for ARM
`sudo apt-get install pkg-config libssl-dev`

## 2. Set Up a Cross-Compilation Sysroot

### Install the Raspberry Pi cross-compilation toolchain
`sudo apt-get install crossbuild-essential-armhf`

### Get the Raspberry Pi sysroot (for ARMv7)
```console
sudo mkdir -p /opt/cross/armv7
sudo chown -R ugo+rwx /opt/cross/armv7
cd /opt/cross/armv7
wget https://github.com/raspberrypi/tools/archive/master.zip
unzip master.zip
```

## 3. Download OpenSSL Source
```console
wget https://codeload.github.com/openssl/openssl/zip/refs/tags/openssl-3.0.2
unzip openssl-openssl-3.0.2.zip
cd openssl-openssl-3.0.2
```

## 4. Configure OpenSSL for ARM
Use the cross-compilation tools from the tools-master directory. Assuming you’re targeting ARMv7:

`./Configure linux-armv4 shared --prefix=/opt/cross/armv7/openssl-armhf --cross-compile-prefix=/opt/cross/armv7/tools-master/arm-bcm2708/arm-linux-gnueabihf/bin/arm-linux-gnueabihf-
`
`--prefix`: Where OpenSSL will be installed after compilation.
`--cross-compile-prefix`: Path to the ARM cross-compilation tools.
## 5. Build and Install OpenSSL for ARM
```console
make
make install
```
This installs the compiled OpenSSL libraries and headers in /opt/cross/armv7/openssl-armhf.

## 6. Set Up Environment Variables

```console
export PKG_CONFIG_PATH=/opt/cross/armv7/openssl-armhf/lib/pkgconfig
export OPENSSL_DIR=/opt/cross/armv7/openssl-armhf
export OPENSSL_LIB_DIR=${OPENSSL_DIR}/lib
export OPENSSL_INCLUDE_DIR=${OPENSSL_DIR}/include
```

## 7. Verify OpenSSL Setup
Check that the compiled OpenSSL libraries and pkg-config work for ARM:

`pkg-config --libs --cflags openssl`

Expected output (with paths specific to your setup):

`-I/opt/cross/armv7/openssl-armhf/include -L/opt/cross/armv7/openssl-armhf/lib -lssl -lcrypto`

## 8. Intall compiler and lineker for ARM
`sudo apt install gcc-arm-linux-gnueabihf`

In order to instruct cargo to use newly installed compiler and linker for ARM, we need to modify project's file `/.cargo/config` with folowing lines:
```
[target.armv7-unknown-linux-gnueabihf]
linker = "arm-linux-gnueabihf-gcc"
```

## 8. Build the Application with Cross-Compilation
`cargo build --release --target=armv7-unknown-linux-gnueabihf`

## 9. Installation package creation
If you want to distribute as .deb package, copy `ngrokmonitor` from `target/armv7-unknown-linux-gnueabihf/release` to `ngrokmonitor/usr/local/bin/` directory inside of the project.

You can also modify the `ngrokmonitor/etc/ngrokmonitor.cfg` config file with actual credentials and email address.

Use dpkg-deb to build the .deb package: `dpkg-deb --build ngrokmonitor`