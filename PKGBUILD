# Maintainer: Gaurav Atreya <allmanpride@gmail.com>
pkgname=string-template-plus
pkgver=0.4.2
pkgrel=1
pkgdesc="String Template with extra functionalities"
arch=('x86_64')
license=('GPL3')
depends=('gcc-libs')
makedepends=('rust' 'cargo')

build() {
	cargo build --release
}

package() {
    cd "$srcdir"
    mkdir -p "$pkgdir/usr/bin"
    cp "../target/release/stp-visualize" "$pkgdir/usr/bin/stp-visualize"
}
