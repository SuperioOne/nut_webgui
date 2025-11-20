pkgname=nut_webgui-bin
pkgver=__PLACEHOLDER_NUTWG_VERSION
pkgrel=1
pkgdesc='Light weight web interface for Network UPS Tools.'
url='https://github.com/SuperioOne/nut_webgui/'
arch=('x86_64' 'aarch64' 'armv7h')
license=('Apache-2.0')
source_x86_64=("nut_webgui_${pkgver}_x86-64-gnu.tar.gz"::"https://github.com/SuperioOne/nut_webgui/releases/download/v${pkgver}/nut_webgui_${pkgver}_x86-64-gnu.tar.gz")
source_aarch64=("nut_webgui_${pkgver}_aarch64-gnu.tar.gz"::"https://github.com/SuperioOne/nut_webgui/releases/download/v${pkgver}/nut_webgui_${pkgver}_aarch64-gnu.tar.gz")
source_armv7h=("nut_webgui_${pkgver}_armv7-musleabi.tar.gz"::"https://github.com/SuperioOne/nut_webgui/releases/download/v${pkgver}/nut_webgui_${pkgver}_armv7-musleabi.tar.gz")
sha256sums_x86_64=('__PLACEHOLDER_NUTWG_SHA256_x86_64')
sha256sums_aarch64=('__PLACEHOLDER_NUTWG_SHA256_AARCH64')
sha256sums_armv7h=('__PLACEHOLDER_NUTWG_SHA256_ARMV7')

package() {
  if [ "$CARCH" = 'x86_64' ]; then
         root_dir="nut_webgui_${pkgver}_x86-64-gnu";
  elif [ "$CARH" = 'aarch64' ]; then
        root_dir="nut_webgui_${pkgver}_aarch64-gnu";
  elif [ "$CARCH" = 'armv7h' ]; then
        root_dir="nut_webgui_${pkgver}_armv7-musleabi";
  fi

  install -Dm 755 "${srcdir}/${root_dir}/nut_webgui" "${pkgdir}/usr/bin/nut_webgui"

  install -dm 744 "${pkgdir}/etc/nut_webgui"
  echo 'version="1"' > "${pkgdir}/etc/nut_webgui/config.toml"
  chmod 744 "${pkgdir}/etc/nut_webgui/config.toml"

  install -dm 755 "${pkgdir}/usr/lib/systemd/system"
    cat <<EOF > "${pkgdir}/usr/lib/systemd/system/nut_webgui.service"
[Unit]
Description=nut_webgui - Simple NUT Web interface
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
ExecStart=/usr/bin/nut_webgui --config-file /etc/nut_webgui/config.toml --allow-env
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF

  chmod 755 "${pkgdir}/usr/lib/systemd/system/nut_webgui.service"
}
