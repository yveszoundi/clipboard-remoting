#!/usr/bin/env sh
set -x

PREVIOUSDIR="$(echo $PWD)"
SCRIPTDIR="$(realpath $(dirname "$0"))"
PROJECTDIR="$(realpath ${SCRIPTDIR}/../..)"
APPVERSION=$(awk -F ' = ' '$1 ~ /version/ { gsub(/[\"]/, "", $2); printf("%s",$2) }' ${PROJECTDIR}/rclip_client/Cargo.toml)
ARTIFACTSDIR="${PROJECTDIR}/artifacts/rclip-linux-amd64-${APPVERSION}"

mkdir -p ${ARTIFACTSDIR}

test -d ${PROJECTDIR}/rclip_config/target && rm -rf ${PROJECTDIR}/rclip_config/target
test -d ${PROJECTDIR}/rclip_client/target && rm -rf ${PROJECTDIR}/rclip_client/target
test -d ${PROJECTDIR}/rclip_server/target && rm -rf ${PROJECTDIR}/rclip_server/target

cd ${PROJECTDIR}

echo "Building rclip_client"
podman run --rm --privileged -v "${PROJECTDIR}":/src -v "${PROJECTDIR}/artifacts":/artifacts docker.io/uycyjnzgntrn/rust-centos8:1.63.0 /bin/bash -c "ln -sf /usr/lib64/libfuse.so.2.9.2 /usr/lib/libfuse.so.2 && mkdir -p /tmp/appdir-gui/usr/bin /tmp/appdir-gui/usr/share/icons /tmp/appdir-cli/usr/bin /tmp/appdir-cli/usr/share/icons && cp /src/ci_cd/linux/xdg/AppRun-gui /tmp/appdir-gui/AppRun && cp /src/ci_cd/linux/xdg/*gui.desktop /tmp/appdir-gui/ && cp /src/ci_cd/linux/xdg/AppRun-cli /tmp/appdir-cli/AppRun && cp /src/ci_cd/linux/xdg/*cli.desktop /tmp/appdir-cli/ && cd /src/rclip_client && /root/.cargo/bin/cargo build --release && cp target/release/rclip-client-gui /tmp/appdir-gui/ && cp target/release/rclip-client-cli /tmp/appdir-cli/ && cp /src/images/Rclip.png /tmp/appdir-gui/usr/share/icons/rclip-client-gui.png && cp /src/images/Rclip.png /tmp/appdir-cli/usr/share/icons/rclip-client-cli.png && ARCH=x86_64 linuxdeploy --appdir /tmp/appdir-gui --desktop-file /tmp/appdir-gui/rclip-client-gui.desktop --icon-filename /tmp/appdir/usr/share/icons/rclip-client-gui.png --output appimage && mv *.AppImage /artifacts/rclip-linux-amd64-${APPVERSION}/rclip-client-gui && ARCH=x86_64 linuxdeploy --appdir /tmp/appdir-cli --desktop-file /tmp/appdir-cli/rclip-client-cli.desktop --icon-filename /tmp/appdir/usr/share/icons/rclip-client-cli.png --output appimage && mv *.AppImage /artifacts/rclip-linux-amd64-${APPVERSION}/rclip-client-cli"

retVal=$?
if [ $retVal -ne 0 ]; then
	echo "Failure to create Linux GUI AppImage binary"
  exit 1
fi

echo "Building rclip_server"
cd ${PROJECTDIR}
podman run --rm --volume "${PWD}":/root/src --workdir /root/src docker.io/joseluisq/rust-linux-darwin-builder:1.60.0 sh -c "RUSTFLAGS='-C target-feature=+crt-static' cargo build --release --target x86_64-unknown-linux-musl --manifest-path /root/src/rclip_server/Cargo.toml"

retVal=$?
if [ $retVal -ne 0 ]; then
	echo "Failure to build other Linux CLI binaries"
  exit 1
fi

cp ${PROJECTDIR}/rclip_server/target/x86_64-unknown-linux-musl/release/rclip-server ${ARTIFACTSDIR}

cp ${SCRIPTDIR}/release_README.txt ${ARTIFACTSDIR}/README.txt

cd ${ARTIFACTSDIR}/.. && tar cvf rclip-linux-amd64-${APPVERSION}.tar rclip-linux-amd64-${APPVERSION}

cd ${SCRIPTDIR}
