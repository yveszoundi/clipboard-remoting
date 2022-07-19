#!/usr/bin/env sh
set -x

PREVIOUSDIR="$(echo $PWD)"
SCRIPTDIR="$(realpath $(dirname "$0"))"
PROJECTDIR="$(realpath ${SCRIPTDIR}/../..)"
APPVERSION=$(awk -F ' = ' '$1 ~ /version/ { gsub(/[\"]/, "", $2); printf("%s",$2) }' ${PROJECTDIR}/rclip_client/Cargo.toml)
ARTIFACTSDIR="${PROJECTDIR}/artifacts/rclip-windows-amd64-${APPVERSION}"

mkdir -p ${ARTIFACTSDIR}

cp ${PROJECTDIR}/LICENSE ${ARTIFACTSDIR}/LICENSE.txt

rm -rf ${PROJECTDIR}/rclip_config/target
rm -rf ${PROJECTDIR}/rclip_client/target
rm -rf ${PROJECTDIR}/rclip_server/target

cd ${PROJECTDIR}

echo "Building all Windows binaries"
podman run --rm --privileged -v "${PROJECTDIR}":/src docker.io/uycyjnzgntrn/rust-windows:1.60.0 sh -c "cd /src/rclip_client && RUSTFLAGS='-C target-feature=+crt-static' cargo build --release --target x86_64-pc-windows-gnu && cd /src/rclip_server && RUSTFLAGS='-C target-feature=+crt-static' cargo build --release --target x86_64-pc-windows-gnu"
retVal=$?
if [ $retVal -ne 0 ]; then
	echo "Failure"
  exit 1
fi

cp ${PROJECTDIR}/rclip_client/target/x86_64-pc-windows-gnu/release/rclip-client-cli.exe ${ARTIFACTSDIR}/
cp ${PROJECTDIR}/rclip_client/target/x86_64-pc-windows-gnu/release/rclip-client-gui.exe ${ARTIFACTSDIR}/
cp ${PROJECTDIR}/rclip_server/target/x86_64-pc-windows-gnu/release/rclip-server.exe ${ARTIFACTSDIR}/


#cp ${SCRIPTDIR}/release_README.txt ${ARTIFACTSDIR}/README.txt

cd ${ARTIFACTSDIR}/.. && zip -r rclip-windows-amd64-${APPVERSION}.zip rclip-windows-amd64-${APPVERSION}

cd ${SCRIPTDIR}
