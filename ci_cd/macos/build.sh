#!/usr/bin/env sh
set -x

PREVIOUSDIR="$(echo $PWD)"
SCRIPTDIR="$(realpath $(dirname "$0"))"
PROJECTDIR="$(realpath ${SCRIPTDIR}/../..)"
APPVERSION=$(awk -F ' = ' '$1 ~ /version/ { gsub(/[\"]/, "", $2); printf("%s",$2) }' ${PROJECTDIR}/rclip_client/Cargo.toml)
ARTIFACTSDIR="${PROJECTDIR}/artifacts/rclip-macos-amd64-${APPVERSION}"

mkdir -p ${ARTIFACTSDIR}

rm -rf ${PROJECTDIR}/rclip_client/target
rm -rf ${PROJECTDIR}/rclip_config/target
rm -rf ${PROJECTDIR}/rclip_server/target

cd ${PROJECTDIR}

echo "Building all Mac OS binaries"
podman run --rm \
    --volume "${PROJECTDIR}":/root/src \
    --workdir /root/src \
    docker.io/joseluisq/rust-linux-darwin-builder:1.60.0 \
    sh -c "export CC=/usr/local/osxcross/target/bin/o64-clang; export CXX=/usr/local/osxcross/target/bin/o64-clang++; cd /root/src/rclip_client && RUSTFLAGS='-C target-feature=+crt-static' cargo build --release --target x86_64-apple-darwin && cd /root/src/rclip_server && RUSTFLAGS='-C target-feature=+crt-static' cargo build --release --target x86_64-apple-darwin"
retVal=$?
if [ $retVal -ne 0 ]; then
	echo "Failure"
  exit 1
fi

cp ${PROJECTDIR}/rclip_client/target/x86_64-apple-darwin/release/rclip-client-cli ${ARTIFACTSDIR}
cp ${PROJECTDIR}/rclip_client/target/x86_64-apple-darwin/release/rclip-client-gui ${ARTIFACTSDIR}
cp ${PROJECTDIR}/rclip_server/target/x86_64-apple-darwin/release/rclip-server  ${ARTIFACTSDIR}

# See https://github.com/zhlynn/zsign
# # See https://forums.ivanti.com/s/article/Obtaining-an-Apple-Developer-ID-Certificate-for-macOS-Provisioning?language=en_US&ui-force-components-controllers-recordGlobalValueProvider.RecordGvp.getRecord=1
# echo "TODO need to create signed app bundle with proper entitlements"

echo "Creating rclip app-bundle"
cd ${SCRIPTDIR}
APPNAME=Rclip
APPBUNDLE=${ARTIFACTSDIR}/${APPNAME}.app
APPBUNDLECONTENTS=${APPBUNDLE}/Contents
APPBUNDLEEXE=${APPBUNDLECONTENTS}/MacOS
APPBUNDLERESOURCES=${APPBUNDLECONTENTS}/Resources
APPBUNDLEICON=${APPBUNDLECONTENTS}/Resources
APPBUNDLECOMPANY="Yves Zoundi"
APPBUNDLEVERSION=${APPVERSION}

mkdir -p ${APPBUNDLE}
mkdir -p ${APPBUNDLE}/Contents
mkdir -p ${APPBUNDLE}/Contents/MacOS
mkdir -p ${APPBUNDLE}/Contents/Resources

convert -scale 16x16   macos/${APPNAME}.png macos/${APPNAME}_16_16.png
convert -scale 32x32   macos/${APPNAME}.png macos/${APPNAME}_32_32.png
convert -scale 128x128 macos/${APPNAME}.png macos/${APPNAME}_128_128.png
convert -scale 256x256 macos/${APPNAME}.png macos/${APPNAME}_256_256.png
convert -scale 512x512 macos/${APPNAME}.png macos/${APPNAME}_512_512.png

cp macos/Info.plist ${APPBUNDLECONTENTS}/
cp macos/PkgInfo ${APPBUNDLECONTENTS}/

png2icns ${APPBUNDLEICON}/${APPNAME}.icns \
         macos/${APPNAME}_16_16.png \
         macos/${APPNAME}_32_32.png \
         macos/${APPNAME}_128_128.png \
         macos/${APPNAME}_256_256.png \
         macos/${APPNAME}_512_512.png

rm macos/${APPNAME}_16_16.png \
   macos/${APPNAME}_32_32.png \
   macos/${APPNAME}_128_128.png \
   macos/${APPNAME}_256_256.png \
   macos/${APPNAME}_512_512.png

cp ${PROJECTDIR}/rclip_client/target/x86_64-apple-darwin/release/rclip-client-cli ${APPBUNDLEEXE}/
mv ${ARTIFACTSDIR}/rclip-client-gui ${APPBUNDLEEXE}/${APPNAME}
perl -pi -e "s/_COMPANY_NAME_/${APPBUNDLECOMPANY}/g" ${APPBUNDLECONTENTS}/Info.plist
perl -pi -e "s/_APPVERSION_/${APPBUNDLEVERSION}/g" ${APPBUNDLECONTENTS}/Info.plist

cp ${SCRIPTDIR}/release_README.txt ${ARTIFACTSDIR}/README.txt

cd ${ARTIFACTSDIR}/.. && zip -r rclip-macos-amd64-${APPVERSION}.zip rclip-macos-amd64-${APPVERSION}

cd ${SCRIPTDIR}
