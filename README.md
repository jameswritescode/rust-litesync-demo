```
wget <litesync-binary.tar.gz>
tar zxf <litesync-binary.tar.gz> -C litesync
cd litesync
sudo ./install
cd ..
SQLITE3_LIB_DIR=$PWD/litesync SQLITE3_INCLUDE_DIR=$PWD/litesync cargo run
```
